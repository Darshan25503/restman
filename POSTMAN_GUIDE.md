# RestMan API - Postman Collection Guide

This guide explains how to use the Postman collection to test the RestMan Auth Service API.

## 📦 Files Included

1. **RestMan_Auth_Service.postman_collection.json** - Complete API collection with all endpoints
2. **RestMan_Local.postman_environment.json** - Environment variables for local development
3. **postman_collection.json** - Simple collection (basic version)

## 🚀 Quick Start

### Step 1: Import Collection

1. Open Postman
2. Click **Import** button (top left)
3. Drag and drop `RestMan_Auth_Service.postman_collection.json`
4. Click **Import**

### Step 2: Import Environment

1. Click **Import** button again
2. Drag and drop `RestMan_Local.postman_environment.json`
3. Click **Import**
4. Select **RestMan - Local Development** environment from the dropdown (top right)

### Step 3: Start the Auth Service

Make sure the auth service is running:

```bash
cd /path/to/restman
cargo run -p auth-service
```

The service should be running on `http://localhost:8001`

## 🧪 Testing Workflow

### For User Role (Customer)

1. **Health Check**
   - Folder: `Health & Status`
   - Request: `Health Check`
   - Expected: `200 OK` with `{"service": "auth-service", "status": "healthy"}`

2. **Request OTP**
   - Folder: `Authentication - User Role`
   - Request: `1. Request OTP (User)`
   - Update `user_email` variable with your email address
   - Click **Send**
   - Expected: `200 OK` with `{"message": "OTP sent to your email"}`
   - Check your email for the 6-digit OTP code

3. **Verify OTP**
   - Request: `2. Verify OTP (User)`
   - Update `user_otp` variable with the code from your email
   - Click **Send**
   - Expected: `200 OK` with session token
   - The session token is automatically saved to `user_session_token` variable

4. **Logout**
   - Request: `3. Logout (User)`
   - Click **Send**
   - Expected: `200 OK` with `{"message": "Logged out successfully"}`
   - Session token is automatically cleared

### For Restaurant Owner Role

Follow the same workflow using the `Authentication - Restaurant Owner` folder:
1. `1. Request OTP (Restaurant)` - Update `rest_email` variable
2. `2. Verify OTP (Restaurant)` - Update `rest_otp` variable
3. `3. Logout (Restaurant)`

### For Kitchen Staff Role

Follow the same workflow using the `Authentication - Kitchen Staff` folder:
1. `1. Request OTP (Kitchen)` - Update `kitch_email` variable
2. `2. Verify OTP (Kitchen)` - Update `kitch_otp` variable
3. `3. Logout (Kitchen)`

## 📝 Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `base_url` | Auth service base URL | `http://localhost:8001` |
| `user_email` | Email for user role | `user@example.com` |
| `user_otp` | OTP code for user | `442946` |
| `user_session_token` | Session token (auto-saved) | `d007b4cc-0c64-48be-9529-5a3f479ca445` |
| `user_id` | User ID (auto-saved) | `3d1a7e86-a7b3-4dd3-9279-4faec0896062` |
| `rest_email` | Email for restaurant owner | `restaurant@example.com` |
| `rest_otp` | OTP code for restaurant owner | `180363` |
| `rest_session_token` | Restaurant session token | Auto-saved |
| `rest_user_id` | Restaurant user ID | Auto-saved |
| `kitch_email` | Email for kitchen staff | `kitchen@example.com` |
| `kitch_otp` | OTP code for kitchen staff | `549911` |
| `kitch_session_token` | Kitchen session token | Auto-saved |
| `kitch_user_id` | Kitchen user ID | Auto-saved |

## 🔧 Updating Variables

### Method 1: Environment Tab
1. Click on **Environments** (left sidebar)
2. Select **RestMan - Local Development**
3. Update the **Current Value** column
4. Click **Save**

### Method 2: Quick Edit
1. Click the **eye icon** (👁️) next to environment dropdown
2. Click **Edit** next to the variable you want to change
3. Update the value
4. Click outside to save

## 🧪 Test Scripts

The collection includes automated test scripts that:

- ✅ Verify response status codes
- ✅ Validate response structure
- ✅ Automatically save session tokens
- ✅ Clear tokens on logout

View test results in the **Test Results** tab after sending a request.

## 📊 Example Responses

### Success Response (Verify OTP)
```json
{
  "token": "d007b4cc-0c64-48be-9529-5a3f479ca445",
  "user_id": "3d1a7e86-a7b3-4dd3-9279-4faec0896062",
  "email": "user@example.com",
  "role": "user"
}
```

### Error Response (Invalid OTP)
```json
{
  "error": "BAD_REQUEST",
  "message": "Bad request: OTP not found or expired"
}
```

## 🔍 Troubleshooting

### Issue: "Could not send request"
- **Solution**: Make sure auth service is running on `http://localhost:8001`
- Check with: `curl http://localhost:8001/api/auth/health`

### Issue: "OTP not found or expired"
- **Solution**: OTP expires after 5 minutes. Request a new OTP
- Make sure you're using the correct email address

### Issue: "Session token not saved"
- **Solution**: Check the **Tests** tab in the request
- Make sure the environment is selected (top right dropdown)

## 🎯 Tips

1. **Use Console**: Open Postman Console (View → Show Postman Console) to see detailed logs
2. **Save Responses**: Click **Save Response** to keep example responses
3. **Organize**: Use folders to organize requests by feature
4. **Share**: Export and share collections with your team

## 📚 API Documentation

For detailed API documentation, see:
- `PHASE_2_COMPLETE.md` - Implementation details
- `SYSTEM_DESIGN.md` - System architecture

## 🔗 Related Services

Once other services are implemented, you can use the session tokens from auth service to authenticate requests to:
- Restaurant Service (port 8002)
- Order Service (port 8003)
- Kitchen Service (port 8004)
- Billing Service (port 8005)
- Analytics Service (port 8006)
- Scheduler Service (port 8007)
- API Gateway (port 8000)

