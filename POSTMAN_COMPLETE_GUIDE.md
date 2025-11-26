# RestMan Complete API - Postman Collection Guide

## 📦 Overview

This Postman collection provides complete API testing for the RestMan microservices platform, including:
- **Auth Service** (Port 8001) - OTP-based authentication
- **Restaurant Service** (Port 8002) - Restaurant and menu management
- **Order Service** (Port 8003) - Order placement and tracking
- **Kitchen Service** (Port 8004) - Kitchen ticket management and email notifications

## 🚀 Quick Start

### 1. Import Files into Postman

Import both files into Postman:
1. **Collection**: `RestMan_Complete_API.postman_collection.json`
2. **Environment**: `RestMan_Complete.postman_environment.json`

### 2. Select Environment

In Postman, select **"RestMan - Complete Environment"** from the environment dropdown (top-right corner).

### 3. Update Your Email

In the environment variables, change `user_email` to your actual email address:
- Click the environment name
- Edit `user_email` value
- Save

### 4. Start Testing!

Follow the workflow below to test the complete system.

---

## 🔄 Complete Testing Workflow

### Phase 1: Authentication

#### Step 1: Request OTP
- **Endpoint**: `Auth Service > Request OTP`
- **Action**: Click "Send"
- **Result**: OTP sent to your email (check inbox/spam)
- **Note**: OTP is valid for 5 minutes

#### Step 2: Verify OTP
- **Endpoint**: `Auth Service > Verify OTP`
- **Action**: 
  1. Check your email for the 6-digit OTP
  2. Update the request body with the OTP code
  3. Click "Send"
- **Result**: 
  - Session token automatically saved to environment
  - User ID automatically saved (used as Owner ID)
- **Session**: Valid for 2 hours

#### Step 3: Validate Session (Optional)
- **Endpoint**: `Auth Service > Validate Session`
- **Action**: Click "Send"
- **Result**: Confirms session is valid

---

### Phase 2: Restaurant Management

#### Step 4: Create Restaurant
- **Endpoint**: `Restaurant Service > Restaurant Management > Create Restaurant`
- **Action**: 
  1. Ensure you're authenticated (user_id is set)
  2. Modify restaurant details in request body if desired
  3. Click "Send"
- **Result**: 
  - Restaurant created
  - Restaurant ID automatically saved to environment
- **Headers**: Uses `X-Owner-Id: {{user_id}}` for ownership

#### Step 5: Get My Restaurants
- **Endpoint**: `Restaurant Service > Restaurant Management > Get My Restaurants`
- **Action**: Click "Send"
- **Result**: List of all restaurants you own

#### Step 6: Get Restaurant Details
- **Endpoint**: `Restaurant Service > Restaurant Management > Get Restaurant Details`
- **Action**: Click "Send"
- **Result**: Full details of the restaurant
- **Note**: This is a public endpoint (no auth required)

---

### Phase 3: Menu Categories

#### Step 7: Create Category
- **Endpoint**: `Restaurant Service > Category Management > Create Category`
- **Action**: 
  1. Modify category details if desired
  2. Click "Send"
- **Result**: 
  - Category created
  - Category ID automatically saved to environment

#### Step 8: Update Category (Optional)
- **Endpoint**: `Restaurant Service > Category Management > Update Category`
- **Action**: Modify details and click "Send"

---

### Phase 4: Food Items

#### Step 9: Create Food Item
- **Endpoint**: `Restaurant Service > Food Management > Create Food Item`
- **Action**: 
  1. Modify food details if desired
  2. Click "Send"
- **Result**: 
  - Food item created
  - Food ID automatically saved to environment

#### Step 10: Update Food Item (Optional)
- **Endpoint**: `Restaurant Service > Food Management > Update Food Item`
- **Action**: Modify details and click "Send"

---

### Phase 5: Menu Retrieval

#### Step 11: Get Full Menu
- **Endpoint**: `Restaurant Service > Menu > Get Full Menu`
- **Action**: Click "Send"
- **Result**: Complete menu with all categories and food items
- **Note**: 
  - This is a public endpoint (no auth required)
  - Response is cached in Redis for 5 minutes
  - Cache is invalidated on any menu update

#### Step 12: Get Foods by IDs (Internal)
- **Endpoint**: `Restaurant Service > Menu > Get Foods by IDs (Internal)`
- **Action**: Click "Send"
- **Result**: Food items by ID (used by Order Service)

---

### Phase 6: Order Management

#### Step 13: Create Order
- **Endpoint**: `Order Service > Create Order`
- **Action**:
  1. Ensure you have completed authentication (session_token set)
  2. Ensure you have created a restaurant and food items
  3. Click "Send"
- **Result**:
  - Order created with status "PLACED"
  - Order ID automatically saved to environment
  - Total amount calculated from food prices
  - Kafka event `order.placed` published
- **Note**:
  - Uses `user_id` from auth verification
  - Uses `restaurant_id` and `food_id` from previous steps
  - Fetches current food prices from Restaurant Service

#### Step 14: Get Order Details
- **Endpoint**: `Order Service > Get Order Details`
- **Action**: Click "Send"
- **Result**: Complete order details with items

#### Step 15: List User Orders
- **Endpoint**: `Order Service > List User Orders`
- **Action**: Click "Send"
- **Result**: All orders for the authenticated user

#### Step 16: Update Order Status
- **Endpoints**:
  - `Order Service > Update Order Status - Accept` (PLACED → ACCEPTED)
  - `Order Service > Update Order Status - In Progress` (ACCEPTED → IN_PROGRESS)
  - `Order Service > Update Order Status - Ready` (IN_PROGRESS → READY)
  - `Order Service > Update Order Status - Completed` (READY → COMPLETED)
  - `Order Service > Update Order Status - Cancelled` (Any → CANCELLED)
- **Action**: Click "Send" on each endpoint in sequence
- **Result**:
  - Order status updated
  - Kafka event `order.status_updated` published
- **Note**:
  - Status transitions are validated
  - Invalid transitions will return 400 error

---

### Phase 7: Cleanup (Optional)

#### Delete Food Item
- **Endpoint**: `Restaurant Service > Food Management > Delete Food Item`

#### Delete Category
- **Endpoint**: `Restaurant Service > Category Management > Delete Category`

#### Delete Restaurant
- **Endpoint**: `Restaurant Service > Restaurant Management > Delete Restaurant`
- **Note**: This is a soft delete (sets is_active = false)

#### Logout
- **Endpoint**: `Auth Service > Logout`
- **Result**: Session invalidated

---

## 📋 Environment Variables

The environment automatically manages these variables:

| Variable | Description | Auto-populated |
|----------|-------------|----------------|
| `auth_base_url` | Auth service URL | No (default: http://localhost:8001) |
| `restaurant_base_url` | Restaurant service URL | No (default: http://localhost:8002) |
| `order_base_url` | Order service URL | No (default: http://localhost:8003) |
| `user_email` | Your email for auth | No (you must set this) |
| `session_token` | Session token from auth | Yes (after OTP verification) |
| `user_id` | User ID (used as Owner ID) | Yes (after OTP verification) |
| `restaurant_id` | Created restaurant ID | Yes (after creating restaurant) |
| `category_id` | Created category ID | Yes (after creating category) |
| `food_id` | Created food item ID | Yes (after creating food) |
| `order_id` | Created order ID | Yes (after creating order) |

---

## 🎯 Testing Tips

### 1. Sequential Testing
Follow the workflow in order - each step builds on the previous one.

### 2. Check Console
Open Postman Console (View > Show Postman Console) to see:
- Auto-saved variables
- Request/response logs
- Debug information

### 3. Ownership Verification
All write operations (create, update, delete) verify ownership using the `X-Owner-Id` header.

### 4. Caching Behavior
The menu endpoint is cached in Redis:
- **TTL**: 5 minutes (300 seconds)
- **Cache Key**: `menu:{restaurant_id}`
- **Invalidation**: Automatic on any menu update (restaurant, category, or food changes)
- **Test**: Call "Get Full Menu" twice - second call should be faster (served from cache)

### 5. Event Publishing
All operations publish events to Kafka:
- **Menu Events**:
  - **Topic**: `menu.events`
  - **Events**: restaurant.created, restaurant.updated, menu.category_created, menu.food_created, etc.
- **Order Events**:
  - **Topic**: `order.events`
  - **Events**: order.placed, order.status_updated
- **Check**: View Kafka UI at http://localhost:8080 to see events

### 6. Order Status Flow
Orders follow a specific status flow:
```
PLACED → ACCEPTED → IN_PROGRESS → READY → COMPLETED
                                        ↓
                                   CANCELLED
```
- Invalid transitions will return a 400 error
- Example: Cannot go from PLACED directly to COMPLETED

---

## 🔍 API Endpoints Reference

### Auth Service (Port 8001)

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| GET | `/api/auth/health` | Health check | No |
| POST | `/api/auth/request-otp` | Request OTP via email | No |
| POST | `/api/auth/verify-otp` | Verify OTP and get session | No |
| POST | `/api/auth/validate` | Validate session token | No |
| POST | `/api/auth/logout` | Invalidate session | No |

### Restaurant Service (Port 8002)

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| GET | `/api/health` | Health check | No |
| POST | `/api/restaurants` | Create restaurant | Yes (X-Owner-Id) |
| GET | `/api/restaurants/my` | Get my restaurants | Yes (X-Owner-Id) |
| GET | `/api/restaurants/{id}` | Get restaurant details | No |
| PUT | `/api/restaurants/{id}` | Update restaurant | Yes (X-Owner-Id) |
| DELETE | `/api/restaurants/{id}` | Delete restaurant | Yes (X-Owner-Id) |
| POST | `/api/restaurants/{id}/categories` | Create category | Yes (X-Owner-Id) |
| PUT | `/api/restaurants/{id}/categories/{cat_id}` | Update category | Yes (X-Owner-Id) |
| DELETE | `/api/restaurants/{id}/categories/{cat_id}` | Delete category | Yes (X-Owner-Id) |
| POST | `/api/restaurants/{id}/foods` | Create food item | Yes (X-Owner-Id) |
| PUT | `/api/restaurants/{id}/foods/{food_id}` | Update food item | Yes (X-Owner-Id) |
| DELETE | `/api/restaurants/{id}/foods/{food_id}` | Delete food item | Yes (X-Owner-Id) |
| GET | `/api/restaurants/{id}/menu` | Get full menu | No |
| GET | `/internal/foods?ids=...` | Get foods by IDs | No (internal) |

### Order Service (Port 8003)

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| GET | `/api/health` | Health check | No |
| POST | `/api/orders` | Create new order | Yes (X-User-Id) |
| GET | `/api/orders/{id}` | Get order details | Yes (X-User-Id) |
| GET | `/api/orders` | List user's orders | Yes (X-User-Id) |
| PATCH | `/api/orders/{id}/status` | Update order status | No |

**Order Status Values**: `PLACED`, `ACCEPTED`, `IN_PROGRESS`, `READY`, `COMPLETED`, `CANCELLED`

---

## 🐛 Troubleshooting

### Issue: "Missing or invalid X-Owner-Id header"
**Solution**: Make sure you've completed the authentication flow first. The `user_id` variable must be set.

### Issue: "Restaurant not found or access denied"
**Solution**: You can only modify restaurants you own. Check that the `restaurant_id` belongs to your `user_id`.

### Issue: "OTP expired"
**Solution**: OTPs are valid for 5 minutes. Request a new OTP and verify it quickly.

### Issue: "Session expired"
**Solution**: Sessions are valid for 2 hours. Re-authenticate by requesting and verifying a new OTP.

### Issue: "Service unavailable"
**Solution**:
1. Check that Docker containers are running: `docker-compose ps`
2. Check that services are running:
   - Auth Service: `curl http://localhost:8001/api/auth/health`
   - Restaurant Service: `curl http://localhost:8002/api/health`
   - Order Service: `curl http://localhost:8003/api/health`

### Issue: "Invalid status transition"
**Solution**: Orders must follow the valid status flow. Check the current order status and ensure you're transitioning to a valid next status.

### Issue: "Food not available"
**Solution**: The food item must have `is_available = true` in the Restaurant Service. Update the food item to make it available.

### Issue: "External service error"
**Solution**: Order Service depends on Restaurant Service. Ensure Restaurant Service is running and accessible at http://localhost:8002.

---

## 📊 Example Responses

### Create Restaurant Response
```json
{
  "id": "742d1373-5e87-4211-9415-a87490777437",
  "owner_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Pizza Palace",
  "description": "Best pizza in town",
  "address": "123 Main St, New York, NY",
  "phone": "+1-555-1234",
  "is_active": true,
  "created_at": "2025-11-25T06:48:01.435088Z",
  "updated_at": "2025-11-25T06:48:01.435088Z"
}
```

### Get Full Menu Response
```json
{
  "restaurant": {
    "id": "742d1373-5e87-4211-9415-a87490777437",
    "name": "Pizza Palace",
    "description": "Best pizza in town",
    ...
  },
  "categories": [
    {
      "id": "cf5a9682-b199-481f-b0a3-a5067eab8626",
      "name": "Pizzas",
      "description": "Our delicious pizzas",
      "display_order": 1,
      "foods": [
        {
          "id": "62e3539c-2dbb-4c49-ae7b-3afb914a56f1",
          "name": "Margherita Pizza",
          "description": "Classic pizza with tomato sauce, mozzarella, and basil",
          "price": "12.99",
          "is_available": true,
          ...
        }
      ]
    }
  ]
}
```

---

## 🎓 Advanced Usage

### Testing Cache Invalidation
1. Get the menu (cached)
2. Update a food item
3. Get the menu again (cache invalidated, fresh data)

### Testing Ownership
1. Create a restaurant with one user
2. Try to update it with a different `X-Owner-Id`
3. Should receive "access denied" error

### Bulk Testing
Use Postman's Collection Runner to run all requests in sequence automatically.

---

## 📝 Notes

- All timestamps are in UTC (ISO 8601 format)
- Prices are stored as DECIMAL for precision
- UUIDs are used for all IDs
- Soft deletes are used for restaurants (is_active flag)
- Hard deletes are used for categories and foods

---

---

## 🍳 Phase 4: Kitchen Service

The Kitchen Service manages kitchen tickets and sends email notifications when orders are ready.

### Step 1: Check Kitchen Service Health
- **Endpoint**: `Kitchen Service > Health Check`
- **Action**: Click "Send"
- **Result**: Returns service status

### Step 2: List All Kitchen Tickets
- **Endpoint**: `Kitchen Service > List All Kitchen Tickets`
- **Action**: Click "Send"
- **Result**: Returns all kitchen tickets
- **Note**: Tickets are automatically created when orders are placed (via Kafka events)

### Step 3: Filter Tickets by Status
- **Endpoint**: `Kitchen Service > List Kitchen Tickets by Status`
- **Action**:
  1. Modify the `status` query parameter (NEW, ACCEPTED, IN_PROGRESS, READY, DELIVERED_TO_SERVICE)
  2. Click "Send"
- **Result**: Returns filtered tickets

### Step 4: Get Specific Kitchen Ticket
- **Endpoint**: `Kitchen Service > Get Kitchen Ticket by ID`
- **Action**:
  1. Set `kitchen_ticket_id` in environment (or it will be auto-populated)
  2. Click "Send"
- **Result**: Returns ticket details
- **Auto-saved**: `kitchen_ticket_id`

### Step 5: Update Ticket Status - Accept Order
- **Endpoint**: `Kitchen Service > Update Ticket Status - ACCEPTED`
- **Action**: Click "Send"
- **Result**: Ticket status changes from NEW → ACCEPTED
- **Note**: Publishes `order.status_updated` event to Kafka

### Step 6: Update Ticket Status - Start Preparation
- **Endpoint**: `Kitchen Service > Update Ticket Status - IN_PROGRESS`
- **Action**: Click "Send"
- **Result**: Ticket status changes from ACCEPTED → IN_PROGRESS

### Step 7: Update Ticket Status - Mark as Ready (Triggers Email!)
- **Endpoint**: `Kitchen Service > Update Ticket Status - READY`
- **Action**: Click "Send"
- **Result**:
  - Ticket status changes from IN_PROGRESS → READY
  - **Email notification sent to user** 📧
  - Email includes order ID and restaurant name
- **Note**: Check the user's email inbox for the notification!

### Step 8: Update Ticket Status - Delivered to Service
- **Endpoint**: `Kitchen Service > Update Ticket Status - DELIVERED_TO_SERVICE`
- **Action**: Click "Send"
- **Result**: Ticket status changes from READY → DELIVERED_TO_SERVICE

---

## 📧 Email Notification Feature

When a kitchen ticket status is updated to **READY**, the system automatically:
1. Fetches the user's email from the Auth Service
2. Sends an HTML email notification with:
   - Subject: "Your Order is Ready!"
   - Order ID
   - Restaurant name
   - Professional HTML formatting
3. Email is sent asynchronously (doesn't block the API response)

**To test email notifications:**
1. Make sure SMTP is configured in the `.env` file
2. Create an order (Order Service)
3. Wait for kitchen ticket to be created automatically
4. Update ticket status through the workflow: NEW → ACCEPTED → IN_PROGRESS → READY
5. Check the user's email inbox for the notification

---

## 🔄 Complete End-to-End Workflow

Here's the complete flow from authentication to order delivery:

1. **Auth Service**: Request OTP → Verify OTP (get session token)
2. **Restaurant Service**: Create Restaurant → Create Category → Create Food Item
3. **Order Service**: Create Order (auto-saves order_id)
4. **Kitchen Service** (automatic): Kitchen ticket created via Kafka event
5. **Kitchen Service**: Accept ticket (NEW → ACCEPTED)
6. **Kitchen Service**: Start preparation (ACCEPTED → IN_PROGRESS)
7. **Kitchen Service**: Mark as ready (IN_PROGRESS → READY) → **Email sent!** 📧
8. **Kitchen Service**: Deliver to service (READY → DELIVERED_TO_SERVICE)

---

## 📊 Environment Variables

The collection uses these auto-populated variables:

| Variable | Description | Auto-populated By |
|----------|-------------|-------------------|
| `session_token` | Authentication token | Verify OTP |
| `user_id` | User UUID | Verify OTP |
| `restaurant_id` | Restaurant UUID | Create Restaurant |
| `category_id` | Category UUID | Create Category |
| `food_id` | Food item UUID | Create Food Item |
| `order_id` | Order UUID | Create Order |
| `kitchen_ticket_id` | Kitchen ticket UUID | Get Kitchen Ticket |

---

## 🚀 Next Steps

After testing the Kitchen Service, you can:
1. Test the Billing Service (coming in Phase 6)
2. View Kafka events in Kafka UI (http://localhost:8080)
3. View analytics in ClickHouse
4. Monitor email delivery logs

---

**Happy Testing! 🎉**


