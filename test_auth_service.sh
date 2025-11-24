#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================="
echo "  Auth Service Testing Script"
echo "========================================="
echo ""

# Test 1: Health Check
echo -e "${YELLOW}Test 1: Health Check${NC}"
RESPONSE=$(curl -s http://localhost:8001/api/auth/health)
echo "Response: $RESPONSE"
if echo "$RESPONSE" | grep -q "healthy"; then
    echo -e "${GREEN}✓ Health check passed${NC}"
else
    echo -e "${RED}✗ Health check failed${NC}"
fi
echo ""

# Test 2: Request OTP (will fail due to SMTP, but we can check Redis)
echo -e "${YELLOW}Test 2: Request OTP${NC}"
EMAIL="test@example.com"
echo "Requesting OTP for: $EMAIL"

# Since SMTP is not configured, let's manually create an OTP in Redis for testing
echo "Manually creating OTP in Redis for testing..."
OTP="123456"
docker exec restman-redis redis-cli SET "otp:$EMAIL" "$OTP" EX 300
echo "OTP set in Redis: $OTP (expires in 300 seconds)"
echo -e "${GREEN}✓ OTP created in Redis${NC}"
echo ""

# Test 3: Verify OTP
echo -e "${YELLOW}Test 3: Verify OTP${NC}"
VERIFY_RESPONSE=$(curl -s -X POST http://localhost:8001/api/auth/verify-otp \
  -H "Content-Type: application/json" \
  -d "{\"email\": \"$EMAIL\", \"code\": \"$OTP\", \"role\": \"user\"}")

echo "Response: $VERIFY_RESPONSE"

if echo "$VERIFY_RESPONSE" | grep -q "token"; then
    echo -e "${GREEN}✓ OTP verification passed${NC}"
    
    # Extract token
    TOKEN=$(echo "$VERIFY_RESPONSE" | jq -r '.token')
    USER_ID=$(echo "$VERIFY_RESPONSE" | jq -r '.user_id')
    echo "Session Token: $TOKEN"
    echo "User ID: $USER_ID"
    
    # Check if session exists in Redis
    echo ""
    echo "Checking session in Redis..."
    SESSION_DATA=$(docker exec restman-redis redis-cli GET "session:$TOKEN")
    if [ -n "$SESSION_DATA" ]; then
        echo -e "${GREEN}✓ Session found in Redis${NC}"
        echo "Session Data: $SESSION_DATA"
    else
        echo -e "${RED}✗ Session not found in Redis${NC}"
    fi
    
    # Test 4: Logout
    echo ""
    echo -e "${YELLOW}Test 4: Logout${NC}"
    LOGOUT_RESPONSE=$(curl -s -X POST http://localhost:8001/api/auth/logout \
      -H "Content-Type: application/json" \
      -d "{\"session_id\": \"$TOKEN\"}")
    
    echo "Response: $LOGOUT_RESPONSE"
    
    if echo "$LOGOUT_RESPONSE" | grep -q "Logged out successfully"; then
        echo -e "${GREEN}✓ Logout passed${NC}"
        
        # Verify session is deleted from Redis
        echo ""
        echo "Verifying session deleted from Redis..."
        SESSION_DATA=$(docker exec restman-redis redis-cli GET "session:$TOKEN")
        if [ -z "$SESSION_DATA" ]; then
            echo -e "${GREEN}✓ Session successfully deleted from Redis${NC}"
        else
            echo -e "${RED}✗ Session still exists in Redis${NC}"
        fi
    else
        echo -e "${RED}✗ Logout failed${NC}"
    fi
else
    echo -e "${RED}✗ OTP verification failed${NC}"
fi

echo ""
echo "========================================="
echo "  Testing Complete"
echo "========================================="

# Check database for user
echo ""
echo -e "${YELLOW}Checking database for created user...${NC}"
docker exec restman-cockroachdb ./cockroach sql --insecure --database=restman_db \
  --execute="SELECT id, email, role, is_active, last_verified_at FROM auth.users WHERE email = '$EMAIL';"

