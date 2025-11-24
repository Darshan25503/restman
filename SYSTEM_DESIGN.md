# Restaurant Management System - Microservices Architecture

## Table of Contents
1. [High-Level Architecture](#high-level-architecture)
2. [Technology Stack](#technology-stack)
3. [Microservices Overview](#microservices-overview)
4. [Database Design](#database-design)
5. [Kafka Event Flows](#kafka-event-flows)
6. [Core Functional Flows](#core-functional-flows)
7. [Analytics Design](#analytics-design)
8. [Implementation Roadmap](#implementation-roadmap)

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         API Gateway (Actix)                      │
│              (Auth Middleware + Role-based Routing)              │
└────────────┬────────────────────────────────────────────────────┘
             │
             ├──────────────────────────────────────────────────┐
             │                                                  │
    ┌────────▼────────┐                              ┌─────────▼────────┐
    │  Auth Service   │                              │ Restaurant Service│
    │  (OTP + JWT)    │                              │ (Menu Management) │
    └────────┬────────┘                              └─────────┬────────┘
             │                                                  │
    ┌────────▼────────┐                              ┌─────────▼────────┐
    │ Scheduler Svc   │                              │  Order Service   │
    │ (2hr Deactivate)│                              │ (Order Creation) │
    └─────────────────┘                              └─────────┬────────┘
                                                               │
                                      ┌────────────────────────┼────────────────┐
                                      │                        │                │
                             ┌────────▼────────┐    ┌─────────▼────────┐  ┌───▼────────┐
                             │ Kitchen Service │    │ Billing Service  │  │ Analytics  │
                             │ (Order Tracking)│    │ (Bill Generation)│  │  Service   │
                             └─────────────────┘    └──────────────────┘  └────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                           Infrastructure Layer                               │
├──────────────────┬──────────────────┬──────────────────┬────────────────────┤
│  CockroachDB     │     Kafka        │      Redis       │    ClickHouse      │
│  (OLTP Data)     │  (Event Bus)     │ (OTP + Session)  │   (Analytics)      │
└──────────────────┴──────────────────┴──────────────────┴────────────────────┘
```

---

## Technology Stack

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **Services** | Rust + Actix Web | All microservices and API Gateway |
| **OLTP Database** | CockroachDB | Transactional data (multi-tenant, distributed) |
| **Event Bus** | Apache Kafka | Async communication between services |
| **Cache/Session** | Redis | OTP storage, session management, menu caching |
| **Analytics DB** | ClickHouse | OLAP queries, reporting, aggregations |
| **Serialization** | Serde + JSON | Data exchange format |
| **Database Client** | SQLx | Async Postgres driver (CockroachDB compatible) |
| **Kafka Client** | rdkafka | Rust Kafka client |

---

## Microservices Overview

### 1. Auth Service
**Responsibility:** User authentication via Email + OTP, session management, 2-hour validity enforcement

**Endpoints:**
- `POST /auth/request-otp` - Generate and send OTP
- `POST /auth/verify-otp` - Verify OTP and create session
- `GET /auth/me` - Get current user info
- `POST /auth/logout` - Invalidate session

**Database (CockroachDB - `auth` schema):**
- `users` - User accounts with roles (user, rest, kitch)
- `otps` - OTP audit trail

**Redis:**
- `otp:{email}` → `{code, expires_at}` (TTL: 5 mins)
- `session:{session_id}` → `{user_id, role, expires_at}` (TTL: 2 hours for user role)

**Kafka Events Emitted:**
- `user.login_success` → `{user_id, role, email, login_at}`
- `user.deactivated` → `{user_id, deactivated_at, reason}`

---

### 2. Scheduler Service
**Responsibility:** Cron job to deactivate users after 2 hours of OTP verification

**Schedule:** Every 10 minutes

**Logic:**
1. Query users where `role='user'` AND `is_active=true` AND `last_verified_at <= now() - 2 hours`
2. Set `is_active=false` for each user
3. Delete sessions from Redis
4. Emit `user.deactivated` event

---

### 3. Restaurant Service
**Responsibility:** Manage restaurants, food categories, and food items

**Endpoints:**
- `POST /restaurants` - Create restaurant (role: rest)
- `GET /restaurants/{id}` - Get restaurant details
- `POST /restaurants/{id}/categories` - Add food category (role: rest)
- `PUT /restaurants/{id}/categories/{cat_id}` - Update category
- `DELETE /restaurants/{id}/categories/{cat_id}` - Delete category
- `POST /restaurants/{id}/foods` - Add food item (role: rest)
- `PUT /restaurants/{id}/foods/{food_id}` - Update food item
- `DELETE /restaurants/{id}/foods/{food_id}` - Delete food item
- `GET /restaurants/{id}/menu` - Get full menu (public)
- `GET /internal/foods?ids=...` - Internal endpoint for Order Service

**Database (CockroachDB - `restaurant` schema):**
- `restaurants` - Restaurant details
- `food_categories` - Menu categories
- `foods` - Food items with pricing

**Redis:**
- `menu:{restaurant_id}` → Full menu JSON (TTL: 5-10 mins)

**Kafka Events Emitted:**
- `menu.category_created/updated/deleted`
- `menu.food_created/updated/deleted`

---

### 4. Order Service
**Responsibility:** Order creation, validation, and status management

**Endpoints:**
- `POST /orders` - Place new order (role: user)
- `GET /orders/{id}` - Get order details
- `GET /orders` - List user's orders
- `PATCH /orders/{id}/status` - Update order status (role: rest)

**Database (CockroachDB - `orders` schema):**
- `orders` - Order header (user, restaurant, status, total)
- `order_items` - Line items with snapshot of food details

**Order Status Flow:**
```
PLACED → ACCEPTED → IN_PROGRESS → READY → COMPLETED
                                        ↓
                                   CANCELLED
```

**Kafka Events:**
- **Emitted:** `order.placed`, `order.status_updated`
- **Consumed:** `order.status_updated` (from Kitchen Service)

---

### 5. Kitchen Service
**Responsibility:** Kitchen-side order tracking and status updates

**Endpoints:**
- `GET /kitchen/tickets` - List all kitchen tickets (role: kitch)
- `GET /kitchen/tickets/{id}` - Get ticket details
- `PATCH /kitchen/tickets/{id}/status` - Update ticket status (role: kitch)

**Database (CockroachDB - `kitchen` schema):**
- `kitchen_tickets` - Kitchen view of orders

**Ticket Status Flow:**
```
NEW → ACCEPTED → IN_PROGRESS → READY → DELIVERED_TO_SERVICE
```

**Kafka Events:**
- **Consumed:** `order.placed` (creates kitchen ticket)
- **Emitted:** `order.status_updated` (when kitchen updates status)

---

### 6. Billing Service
**Responsibility:** Bill generation and payment tracking

**Endpoints:**
- `GET /billing/orders/{order_id}` - Get bill for order
- `POST /billing/orders/{order_id}/finalize` - Finalize and mark as paid
- `PATCH /billing/bills/{id}/payment` - Update payment method

**Database (CockroachDB - `billing` schema):**
- `bills` - Bill details with tax, discount, payment info

**Kafka Events:**
- **Consumed:** `order.placed` (creates provisional bill)
- **Emitted:** `bill.generated`, `bill.paid`

---

### 7. Analytics Service
**Responsibility:** Consume events from Kafka and populate ClickHouse for reporting

**Endpoints:**
- `GET /analytics/top-foods?date=...&period=day|month|year` - Most ordered foods
- `GET /analytics/revenue?period=month&order_type=...` - Revenue analysis
- `GET /analytics/orders-by-status?date=...` - Order status distribution

**ClickHouse Tables:**
- `orders_analytics` - Denormalized order data
- `order_items_analytics` - Line-item level data for food analysis

**Kafka Events Consumed:**
- `order.placed`, `order.status_updated`, `bill.paid`, `menu.*`

---

### 8. API Gateway
**Responsibility:** Single entry point, authentication, authorization, routing

**Features:**
- JWT/Session validation middleware
- Role-based access control (RBAC)
- Route requests to appropriate microservices
- Rate limiting (optional)
- Request/response logging

**Routing Table:**
```
/auth/*        → Auth Service
/restaurants/* → Restaurant Service
/orders/*      → Order Service
/kitchen/*     → Kitchen Service
/billing/*     → Billing Service
/analytics/*   → Analytics Service
```

---

## Database Design

### CockroachDB Schemas

#### Auth Schema
```sql
-- users table
CREATE TABLE auth.users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    role VARCHAR(20) NOT NULL CHECK (role IN ('user', 'rest', 'kitch')),
    is_active BOOLEAN DEFAULT false,
    last_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- otps table
CREATE TABLE auth.otps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL,
    code VARCHAR(10) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    consumed BOOLEAN DEFAULT false,
    purpose VARCHAR(50) DEFAULT 'login',
    created_at TIMESTAMPTZ DEFAULT now()
);
```

#### Restaurant Schema
```sql
-- restaurants table
CREATE TABLE restaurant.restaurants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    address TEXT,
    phone VARCHAR(20),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- food_categories table
CREATE TABLE restaurant.food_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    restaurant_id UUID NOT NULL REFERENCES restaurant.restaurants(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- foods table
CREATE TABLE restaurant.foods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    restaurant_id UUID NOT NULL REFERENCES restaurant.restaurants(id),
    category_id UUID REFERENCES restaurant.food_categories(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price DECIMAL(10, 2) NOT NULL,
    is_available BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);
```

#### Orders Schema
```sql
-- orders table
CREATE TABLE orders.orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    restaurant_id UUID NOT NULL,
    order_type VARCHAR(20) NOT NULL CHECK (order_type IN ('dine_in', 'takeaway', 'delivery')),
    status VARCHAR(20) NOT NULL DEFAULT 'PLACED',
    total_price DECIMAL(10, 2) NOT NULL,
    placed_at TIMESTAMPTZ DEFAULT now(),
    completed_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- order_items table
CREATE TABLE orders.order_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders.orders(id),
    food_id UUID NOT NULL,
    food_name_snapshot VARCHAR(255) NOT NULL,
    unit_price DECIMAL(10, 2) NOT NULL,
    quantity INT NOT NULL,
    line_total DECIMAL(10, 2) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now()
);
```

#### Kitchen Schema
```sql
-- kitchen_tickets table
CREATE TABLE kitchen.kitchen_tickets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL,
    restaurant_id UUID NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'NEW',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);
```

#### Billing Schema
```sql
-- bills table
CREATE TABLE billing.bills (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL,
    user_id UUID NOT NULL,
    restaurant_id UUID NOT NULL,
    subtotal DECIMAL(10, 2) NOT NULL,
    tax_amount DECIMAL(10, 2) DEFAULT 0,
    discount_amount DECIMAL(10, 2) DEFAULT 0,
    total_amount DECIMAL(10, 2) NOT NULL,
    status VARCHAR(20) DEFAULT 'PENDING',
    generated_at TIMESTAMPTZ DEFAULT now(),
    paid_at TIMESTAMPTZ,
    payment_method VARCHAR(20),
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);
```

---

## Kafka Event Flows

### Topic Structure
```
user.events
├── user.login_success
└── user.deactivated

menu.events
├── menu.category_created
├── menu.category_updated
├── menu.category_deleted
├── menu.food_created
├── menu.food_updated
└── menu.food_deleted

order.events
├── order.placed
└── order.status_updated

bill.events
├── bill.generated
└── bill.paid
```

### Event Schemas

#### user.login_success
```json
{
  "event_type": "user.login_success",
  "event_id": "uuid",
  "timestamp": "2025-11-20T10:30:00Z",
  "data": {
    "user_id": "uuid",
    "email": "user@example.com",
    "role": "user",
    "login_at": "2025-11-20T10:30:00Z"
  }
}
```

#### order.placed
```json
{
  "event_type": "order.placed",
  "event_id": "uuid",
  "timestamp": "2025-11-20T12:00:00Z",
  "data": {
    "order_id": "uuid",
    "user_id": "uuid",
    "restaurant_id": "uuid",
    "order_type": "dine_in",
    "total_price": 450.00,
    "items": [
      {
        "food_id": "uuid",
        "food_name": "Margherita Pizza",
        "quantity": 2,
        "unit_price": 200.00,
        "line_total": 400.00
      }
    ],
    "placed_at": "2025-11-20T12:00:00Z"
  }
}
```

#### order.status_updated
```json
{
  "event_type": "order.status_updated",
  "event_id": "uuid",
  "timestamp": "2025-11-20T12:15:00Z",
  "data": {
    "order_id": "uuid",
    "old_status": "PLACED",
    "new_status": "ACCEPTED",
    "changed_by": "kitch",
    "changed_by_user_id": "uuid",
    "changed_at": "2025-11-20T12:15:00Z"
  }
}
```

#### bill.paid
```json
{
  "event_type": "bill.paid",
  "event_id": "uuid",
  "timestamp": "2025-11-20T13:00:00Z",
  "data": {
    "bill_id": "uuid",
    "order_id": "uuid",
    "total_amount": 450.00,
    "payment_method": "card",
    "paid_at": "2025-11-20T13:00:00Z"
  }
}
```

---

## Core Functional Flows

### Flow 1: User Login with OTP (2-Hour Validity)

```
┌──────┐                 ┌──────────────┐              ┌─────────────┐         ┌───────┐
│ User │                 │ Auth Service │              │ CockroachDB │         │ Redis │
└──┬───┘                 └──────┬───────┘              └──────┬──────┘         └───┬───┘
   │                            │                             │                    │
   │ POST /auth/request-otp     │                             │                    │
   │ {email: "user@ex.com"}     │                             │                    │
   ├───────────────────────────>│                             │                    │
   │                            │                             │                    │
   │                            │ Generate OTP (6-digit)      │                    │
   │                            │                             │                    │
   │                            │ INSERT INTO otps            │                    │
   │                            ├────────────────────────────>│                    │
   │                            │                             │                    │
   │                            │ SET otp:{email} (TTL 5min)  │                    │
   │                            ├────────────────────────────────────────────────>│
   │                            │                             │                    │
   │                            │ Send Email (OTP: 123456)    │                    │
   │<───────────────────────────┤                             │                    │
   │ {message: "OTP sent"}      │                             │                    │
   │                            │                             │                    │
   │ POST /auth/verify-otp      │                             │                    │
   │ {email, code: "123456"}    │                             │                    │
   ├───────────────────────────>│                             │                    │
   │                            │                             │                    │
   │                            │ GET otp:{email}             │                    │
   │                            ├────────────────────────────────────────────────>│
   │                            │<────────────────────────────────────────────────┤
   │                            │ {code: "123456", ...}       │                    │
   │                            │                             │                    │
   │                            │ UPSERT users                │                    │
   │                            │ SET is_active=true          │                    │
   │                            │ SET last_verified_at=now()  │                    │
   │                            ├────────────────────────────>│                    │
   │                            │                             │                    │
   │                            │ Generate session_id         │                    │
   │                            │ SET session:{id} (TTL 2hrs) │                    │
   │                            ├────────────────────────────────────────────────>│
   │                            │                             │                    │
   │                            │ Emit: user.login_success    │                    │
   │                            │ ──> Kafka                   │                    │
   │                            │                             │                    │
   │<───────────────────────────┤                             │                    │
   │ {token, user_id, role}     │                             │                    │
```

**After 2 hours:**
```
┌───────────────────┐              ┌─────────────┐         ┌───────┐
│ Scheduler Service │              │ CockroachDB │         │ Redis │
└─────────┬─────────┘              └──────┬──────┘         └───┬───┘
          │                               │                    │
          │ (Every 10 minutes)            │                    │
          │                               │                    │
          │ SELECT users WHERE            │                    │
          │ is_active=true AND            │                    │
          │ last_verified_at <= now()-2h  │                    │
          ├──────────────────────────────>│                    │
          │<──────────────────────────────┤                    │
          │ [user_id_1, user_id_2, ...]   │                    │
          │                               │                    │
          │ UPDATE users                  │                    │
          │ SET is_active=false           │                    │
          ├──────────────────────────────>│                    │
          │                               │                    │
          │ DEL session:{user_id}:*       │                    │
          ├───────────────────────────────────────────────────>│
          │                               │                    │
          │ Emit: user.deactivated        │                    │
          │ ──> Kafka                     │                    │
```

---

### Flow 2: Restaurant Adds Food Category & Items

```
┌──────────┐           ┌────────────────────┐         ┌─────────────┐        ┌───────┐
│ Rest User│           │ Restaurant Service │         │ CockroachDB │        │ Kafka │
└────┬─────┘           └─────────┬──────────┘         └──────┬──────┘        └───┬───┘
     │                           │                           │                   │
     │ POST /restaurants/{id}/categories                     │                   │
     │ {name: "Pizzas", ...}     │                           │                   │
     ├──────────────────────────>│                           │                   │
     │                           │                           │                   │
     │                           │ Verify role=rest          │                   │
     │                           │ Verify ownership          │                   │
     │                           │                           │                   │
     │                           │ INSERT INTO food_categories                   │
     │                           ├──────────────────────────>│                   │
     │                           │                           │                   │
     │                           │ Emit: menu.category_created                   │
     │                           ├───────────────────────────────────────────────>│
     │                           │                           │                   │
     │                           │ Invalidate cache          │                   │
     │                           │ DEL menu:{restaurant_id}  │                   │
     │                           │                           │                   │
     │<──────────────────────────┤                           │                   │
     │ {category_id, ...}        │                           │                   │
     │                           │                           │                   │
     │ POST /restaurants/{id}/foods                          │                   │
     │ {name: "Margherita", category_id, price: 200}         │                   │
     ├──────────────────────────>│                           │                   │
     │                           │                           │                   │
     │                           │ INSERT INTO foods         │                   │
     │                           ├──────────────────────────>│                   │
     │                           │                           │                   │
     │                           │ Emit: menu.food_created   │                   │
     │                           ├───────────────────────────────────────────────>│
     │                           │                           │                   │
     │<──────────────────────────┤                           │                   │
     │ {food_id, ...}            │                           │                   │
```

---

### Flow 3: User Places Order (Full End-to-End)

```
┌──────┐    ┌─────────┐    ┌──────────────┐    ┌────────────┐    ┌─────────┐    ┌───────┐
│ User │    │ Gateway │    │Order Service │    │  Kitchen   │    │ Billing │    │ Kafka │
└──┬───┘    └────┬────┘    └──────┬───────┘    │  Service   │    │ Service │    └───┬───┘
   │             │                 │            └─────┬──────┘    └────┬────┘        │
   │ POST /orders                  │                  │                │             │
   │ {restaurant_id, items[]}      │                  │                │             │
   ├────────────>│                 │                  │                │             │
   │             │                 │                  │                │             │
   │             │ Verify session  │                  │                │             │
   │             │ (role=user)     │                  │                │             │
   │             │                 │                  │                │             │
   │             │ Forward request │                  │                │             │
   │             ├────────────────>│                  │                │             │
   │             │                 │                  │                │             │
   │             │                 │ Validate foods   │                │             │
   │             │                 │ (call Restaurant Service)         │             │
   │             │                 │                  │                │             │
   │             │                 │ Calculate total  │                │             │
   │             │                 │                  │                │             │
   │             │                 │ INSERT orders    │                │             │
   │             │                 │ INSERT order_items                │             │
   │             │                 │                  │                │             │
   │             │                 │ Emit: order.placed                │             │
   │             │                 ├───────────────────────────────────────────────>│
   │             │                 │                  │                │             │
   │             │                 │                  │<───────────────────────────┤
   │             │                 │                  │ Consume: order.placed       │
   │             │                 │                  │                │             │
   │             │                 │                  │ CREATE kitchen_ticket       │
   │             │                 │                  │                │             │
   │             │                 │                  │                │<────────────┤
   │             │                 │                  │                │ Consume: order.placed
   │             │                 │                  │                │             │
   │             │                 │                  │                │ CREATE bill │
   │             │                 │                  │                │ (status=PENDING)
   │             │                 │                  │                │             │
   │             │<────────────────┤                  │                │             │
   │<────────────┤                 │                  │                │             │
   │ {order_id, status: "PLACED"}  │                  │                │             │
```

---

### Flow 4: Kitchen Updates Order Status

```
┌────────────┐              ┌──────────────┐              ┌───────┐
│   Kitch    │              │   Kitchen    │              │ Kafka │
│    User    │              │   Service    │              └───┬───┘
└─────┬──────┘              └──────┬───────┘                  │
      │                            │                          │
      │ PATCH /kitchen/tickets/{id}/status                    │
      │ {status: "IN_PROGRESS"}    │                          │
      ├───────────────────────────>│                          │
      │                            │                          │
      │                            │ UPDATE kitchen_tickets   │
      │                            │ SET status='IN_PROGRESS' │
      │                            │                          │
      │                            │ Emit: order.status_updated
      │                            ├─────────────────────────>│
      │                            │                          │
      │<───────────────────────────┤                          │
      │ {ticket_id, status}        │                          │
      │                            │                          │

┌──────────────┐                   │
│Order Service │<──────────────────┤
└──────┬───────┘   Consume: order.status_updated
       │
       │ UPDATE orders
       │ SET status='IN_PROGRESS'
```

---

### Flow 5: Bill Finalization & Payment

```
┌──────────┐              ┌─────────────┐              ┌───────┐
│   Rest   │              │   Billing   │              │ Kafka │
│   User   │              │   Service   │              └───┬───┘
└────┬─────┘              └──────┬──────┘                  │
     │                           │                         │
     │ POST /billing/orders/{order_id}/finalize            │
     │ {payment_method: "card"}  │                         │
     ├──────────────────────────>│                         │
     │                           │                         │
     │                           │ UPDATE bills            │
     │                           │ SET status='PAID'       │
     │                           │ SET paid_at=now()       │
     │                           │                         │
     │                           │ Emit: bill.paid         │
     │                           ├────────────────────────>│
     │                           │                         │
     │<──────────────────────────┤                         │
     │ {bill_id, total_amount}   │                         │
```

---

## Analytics Design

### ClickHouse Tables

#### orders_analytics
```sql
CREATE TABLE restman_analytics.orders_analytics (
    event_date Date,
    event_timestamp DateTime,
    order_id String,
    user_id String,
    restaurant_id String,
    order_type String,
    status String,
    total_price Float64,
    placed_at DateTime,
    completed_at DateTime,
    payment_status String,
    payment_method String
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(event_date)
ORDER BY (event_date, restaurant_id, order_id);
```

#### order_items_analytics
```sql
CREATE TABLE restman_analytics.order_items_analytics (
    event_date Date,
    event_timestamp DateTime,
    order_id String,
    restaurant_id String,
    food_id String,
    food_name String,
    category_name String,
    quantity UInt32,
    unit_price Float64,
    line_total Float64,
    order_type String,
    placed_at DateTime
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(event_date)
ORDER BY (event_date, restaurant_id, food_id);
```

---

### Analytics Queries

#### 1. Most Ordered Food Items by Day
```sql
SELECT
    event_date,
    food_name,
    SUM(quantity) AS total_quantity,
    SUM(line_total) AS total_revenue
FROM order_items_analytics
WHERE event_date = '2025-11-20'
GROUP BY event_date, food_name
ORDER BY total_quantity DESC
LIMIT 10;
```

#### 2. Most Ordered Food Items by Month
```sql
SELECT
    toStartOfMonth(event_date) AS month,
    food_name,
    SUM(quantity) AS total_quantity,
    SUM(line_total) AS total_revenue
FROM order_items_analytics
WHERE toYear(event_date) = 2025 AND toMonth(event_date) = 11
GROUP BY month, food_name
ORDER BY total_quantity DESC
LIMIT 10;
```

#### 3. Most Ordered Food Items by Year
```sql
SELECT
    toYear(event_date) AS year,
    food_name,
    SUM(quantity) AS total_quantity,
    SUM(line_total) AS total_revenue
FROM order_items_analytics
WHERE year = 2025
GROUP BY year, food_name
ORDER BY total_quantity DESC
LIMIT 10;
```

#### 4. Revenue by Order Type (Monthly)
```sql
SELECT
    toStartOfMonth(event_date) AS month,
    order_type,
    COUNT(DISTINCT order_id) AS total_orders,
    SUM(line_total) AS total_revenue
FROM order_items_analytics
WHERE toYear(event_date) = 2025
GROUP BY month, order_type
ORDER BY month, order_type;
```

#### 5. Revenue by Restaurant (Daily/Monthly/Yearly)
```sql
-- Daily
SELECT
    event_date,
    restaurant_id,
    COUNT(DISTINCT order_id) AS total_orders,
    SUM(total_price) AS total_revenue
FROM orders_analytics
WHERE event_date >= '2025-11-01' AND event_date <= '2025-11-30'
GROUP BY event_date, restaurant_id
ORDER BY event_date, total_revenue DESC;

-- Monthly
SELECT
    toStartOfMonth(event_date) AS month,
    restaurant_id,
    COUNT(DISTINCT order_id) AS total_orders,
    SUM(total_price) AS total_revenue
FROM orders_analytics
WHERE toYear(event_date) = 2025
GROUP BY month, restaurant_id
ORDER BY month, total_revenue DESC;
```

#### 6. Order Status Distribution
```sql
SELECT
    status,
    COUNT(*) AS count,
    AVG(total_price) AS avg_order_value
FROM orders_analytics
WHERE event_date >= today() - INTERVAL 7 DAY
GROUP BY status;
```

---

### Analytics Service - Kafka Consumer Logic

The Analytics Service consumes events and transforms them into ClickHouse inserts:

**On `order.placed` event:**
```
1. Extract order details
2. Insert into orders_analytics
3. For each item in order.items:
   - Insert into order_items_analytics
```

**On `order.status_updated` event:**
```
1. Update orders_analytics (if using ReplacingMergeTree)
   OR
2. Insert new row with updated status
```

**On `bill.paid` event:**
```
1. Update orders_analytics with payment info
   - payment_status = 'PAID'
   - payment_method = event.payment_method
```

---

## Implementation Roadmap

### Phase 0: Infrastructure Setup (Week 1)

**Goal:** Set up all infrastructure components

**Tasks:**
1. **Docker Compose Setup**
   - Create `docker-compose.yml` with:
     - CockroachDB (single node for dev)
     - Kafka + Zookeeper
     - Redis
     - ClickHouse
   - Test connectivity to all services

2. **CockroachDB Initialization**
   - Create database `restman_db`
   - Create schemas: `auth`, `restaurant`, `orders`, `kitchen`, `billing`
   - Run initial migrations (empty tables for now)

3. **Kafka Topics**
   - Create topics:
     - `user.events`
     - `menu.events`
     - `order.events`
     - `bill.events`

4. **ClickHouse Initialization**
   - Create database `restman_analytics`
   - Create tables: `orders_analytics`, `order_items_analytics`

**Deliverable:** All infrastructure running via `docker-compose up`

---

### Phase 1: Workspace & Shared Libraries (Week 1-2)

**Goal:** Set up Rust workspace and shared code

**Tasks:**
1. **Create Workspace Structure**
   ```
   restman/
   ├── Cargo.toml (workspace)
   ├── docker-compose.yml
   ├── services/
   │   ├── auth-service/
   │   ├── restaurant-service/
   │   ├── order-service/
   │   ├── kitchen-service/
   │   ├── billing-service/
   │   ├── analytics-service/
   │   └── scheduler-service/
   ├── gateway/
   └── shared/
       ├── models/
       ├── kafka-client/
       ├── db-utils/
       └── error-handling/
   ```

2. **Shared Libraries**
   - `shared/models`: Common structs (User, Order, Event schemas)
   - `shared/kafka-client`: Kafka producer/consumer wrappers
   - `shared/db-utils`: SQLx connection pool helpers
   - `shared/error-handling`: Common error types

**Deliverable:** Workspace compiles, shared libraries ready

---

### Phase 2: Auth Service (Week 2-3)

**Goal:** Complete authentication with OTP and 2-hour validity

**Tasks:**
1. **Database Layer**
   - Create migrations for `auth.users` and `auth.otps`
   - Implement repository pattern (UserRepository, OtpRepository)

2. **Redis Layer**
   - OTP storage functions
   - Session management functions

3. **Business Logic**
   - OTP generation (6-digit random)
   - Email sending (stub initially, use `lettre` later)
   - Session creation (generate UUID, store in Redis)

4. **API Endpoints**
   - `POST /auth/request-otp`
   - `POST /auth/verify-otp`
   - `GET /auth/me`
   - `POST /auth/logout`

5. **Kafka Producer**
   - Emit `user.login_success` event
   - Emit `user.deactivated` event (for scheduler)

6. **Testing**
   - Unit tests for OTP generation
   - Integration tests with real DB/Redis (testcontainers)
   - E2E test: request OTP → verify → get session

**Deliverable:** Auth Service running on port 8001, fully tested

---

### Phase 3: Scheduler Service (Week 3)

**Goal:** Deactivate users after 2 hours

**Tasks:**
1. **Cron Logic**
   - Use `tokio::time::interval` for 10-minute intervals
   - Query CockroachDB for expired users
   - Update `is_active = false`
   - Delete sessions from Redis

2. **Kafka Producer**
   - Emit `user.deactivated` events

3. **Testing**
   - Manual test: create user, wait 2 hours (or mock time), verify deactivation

**Deliverable:** Scheduler running as background service

---

### Phase 4: Restaurant Service (Week 4)

**Goal:** Menu management (categories + foods)

**Tasks:**
1. **Database Layer**
   - Migrations for `restaurant.restaurants`, `food_categories`, `foods`
   - Repositories

2. **Business Logic**
   - Ownership verification (only owner can edit their restaurant)
   - Menu validation

3. **API Endpoints**
   - `POST /restaurants`
   - `POST /restaurants/{id}/categories`
   - `POST /restaurants/{id}/foods`
   - `GET /restaurants/{id}/menu`
   - `GET /internal/foods?ids=...` (for Order Service)

4. **Redis Caching**
   - Cache full menu with TTL
   - Invalidate on updates

5. **Kafka Producer**
   - Emit `menu.*` events

6. **Testing**
   - CRUD tests for categories and foods
   - Cache invalidation tests

**Deliverable:** Restaurant Service running on port 8002

---

### Phase 5: Order Service (Week 5-6)

**Goal:** Order creation and management

**Tasks:**
1. **Database Layer**
   - Migrations for `orders.orders`, `order_items`
   - Repositories

2. **Business Logic**
   - Validate user is active (call Auth Service or check token)
   - Validate foods exist (call Restaurant Service)
   - Calculate total price
   - Create order with items (transaction)

3. **API Endpoints**
   - `POST /orders`
   - `GET /orders/{id}`
   - `GET /orders` (list user's orders)
   - `PATCH /orders/{id}/status`

4. **Kafka Producer**
   - Emit `order.placed`
   - Emit `order.status_updated`

5. **Kafka Consumer**
   - Consume `order.status_updated` from Kitchen Service
   - Update local order status

6. **Testing**
   - Place order flow (mock Restaurant Service)
   - Status update flow

**Deliverable:** Order Service running on port 8003

---

### Phase 6: Kitchen Service (Week 6)

**Goal:** Kitchen ticket management

**Tasks:**
1. **Database Layer**
   - Migrations for `kitchen.kitchen_tickets`
   - Repository

2. **Kafka Consumer**
   - Consume `order.placed`
   - Create kitchen ticket

3. **API Endpoints**
   - `GET /kitchen/tickets`
   - `PATCH /kitchen/tickets/{id}/status`

4. **Kafka Producer**
   - Emit `order.status_updated`

5. **Testing**
   - Consume order.placed → create ticket
   - Update ticket status → emit event

**Deliverable:** Kitchen Service running on port 8004

---

### Phase 7: Billing Service (Week 7)

**Goal:** Bill generation and payment

**Tasks:**
1. **Database Layer**
   - Migrations for `billing.bills`
   - Repository

2. **Kafka Consumer**
   - Consume `order.placed`
   - Create provisional bill

3. **API Endpoints**
   - `GET /billing/orders/{order_id}`
   - `POST /billing/orders/{order_id}/finalize`

4. **Kafka Producer**
   - Emit `bill.generated`
   - Emit `bill.paid`

5. **Testing**
   - Bill creation on order placement
   - Bill finalization

**Deliverable:** Billing Service running on port 8005

---

### Phase 8: Analytics Service (Week 8)

**Goal:** Populate ClickHouse for reporting

**Tasks:**
1. **Kafka Consumers**
   - Consume `order.placed` → insert into ClickHouse
   - Consume `order.status_updated` → update ClickHouse
   - Consume `bill.paid` → update payment info

2. **ClickHouse Client**
   - Use `clickhouse-rs` or HTTP client
   - Batch inserts for performance

3. **API Endpoints**
   - `GET /analytics/top-foods`
   - `GET /analytics/revenue`
   - `GET /analytics/orders-by-status`

4. **Testing**
   - Emit test events → verify ClickHouse data
   - Query endpoints

**Deliverable:** Analytics Service running on port 8006

---

### Phase 9: API Gateway (Week 9)

**Goal:** Single entry point with auth/routing

**Tasks:**
1. **Routing**
   - Route `/auth/*` → Auth Service
   - Route `/restaurants/*` → Restaurant Service
   - Route `/orders/*` → Order Service
   - Route `/kitchen/*` → Kitchen Service
   - Route `/billing/*` → Billing Service
   - Route `/analytics/*` → Analytics Service

2. **Auth Middleware**
   - Extract session token from header
   - Validate with Redis
   - Attach user context to request

3. **RBAC Middleware**
   - Check role for protected routes
   - `/restaurants/*/categories` → only `rest`
   - `/kitchen/*` → only `kitch`
   - `/orders` → only `user`

4. **Testing**
   - E2E tests through gateway

**Deliverable:** Gateway running on port 8000

---

### Phase 10: Integration & E2E Testing (Week 10)

**Goal:** Test full system end-to-end

**Test Scenarios:**
1. **User Journey**
   - User requests OTP → verifies → places order → views order
   - Wait 2 hours → verify deactivation → re-login

2. **Restaurant Journey**
   - Rest user creates restaurant → adds categories → adds foods
   - User places order for those foods

3. **Kitchen Journey**
   - Kitchen user views tickets → updates status → order status changes

4. **Billing Journey**
   - Order placed → bill created → bill finalized → payment recorded

5. **Analytics Journey**
   - Place multiple orders → query analytics → verify data

**Deliverable:** All services working together, documented test results

---

## Key Design Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Auth Mechanism** | Session IDs in Redis (for `user` role) | Easy to invalidate for 2-hour requirement |
| **Service Communication** | REST for sync, Kafka for async | Simple, debuggable, scalable |
| **Database** | Shared CockroachDB with schemas | Pragmatic, easier to manage |
| **Kafka Topics** | One topic per service | Fewer topics, filter by event_type |
| **ClickHouse Updates** | Insert new rows (append-only) | Simpler than ReplacingMergeTree |
| **Monorepo vs Polyrepo** | Monorepo (Cargo workspace) | Easier development, shared code |

---

## Next Steps

1. **Review this design** with your team/stakeholders
2. **Set up infrastructure** (Phase 0)
3. **Start with Auth Service** (Phase 2) - it's the foundation
4. **Iterate service by service** following the roadmap
5. **Test continuously** - don't wait until the end

---

## Questions to Consider

1. **Do you need multi-tenancy?** (Multiple restaurants per `rest` user?)
2. **Do you need real-time notifications?** (WebSockets for order updates?)
3. **Do you need file uploads?** (Restaurant logos, food images?)
4. **Do you need search?** (Elasticsearch for menu search?)
5. **Do you need rate limiting?** (Prevent abuse?)
6. **Do you need observability?** (Prometheus + Grafana for metrics?)
7. **Do you need distributed tracing?** (Jaeger/OpenTelemetry?)

These can be added incrementally after the core system is working.

---

**End of System Design Document**

