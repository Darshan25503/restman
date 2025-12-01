# RestMan - Restaurant Management System

A production-ready microservices-based restaurant management platform built with Rust, featuring event-driven architecture, real-time analytics, and comprehensive API gateway.

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    API Gateway (Port 8000)                       │
│         Authentication Middleware + Request Routing              │
└────────────┬────────────────────────────────────────────────────┘
             │
    ┌────────┴────────────────────────────────────────┐
    │                                                  │
┌───▼────────────┐  ┌──────────────┐  ┌─────────────▼──────┐
│ Auth Service   │  │ Restaurant   │  │  Order Service     │
│   Port 8001    │  │   Service    │  │    Port 8003       │
└────────────────┘  │  Port 8002   │  └────────────────────┘
                    └──────────────┘
┌────────────────┐  ┌──────────────┐  ┌────────────────────┐
│ Kitchen Service│  │  Billing     │  │  Analytics Service │
│   Port 8004    │  │  Service     │  │    Port 8006       │
└────────────────┘  │  Port 8005   │  └────────────────────┘
                    └──────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    Infrastructure Layer                          │
├──────────────┬──────────────┬──────────────┬───────────────────┤
│ CockroachDB  │    Kafka     │    Redis     │   ClickHouse      │
│  Port 26257  │  Port 9092   │  Port 6379   │   Port 8124       │
└──────────────┴──────────────┴──────────────┴───────────────────┘
```

## 🚀 Features

### Core Functionality
- **OTP-based Authentication** - Email-based login with 6-digit OTP codes
- **Restaurant Management** - CRUD operations for restaurants, categories, and menu items
- **Order Management** - Complete order lifecycle from creation to delivery
- **Kitchen Operations** - Real-time kitchen ticket management with status updates
- **Billing System** - Automated bill generation with 10% tax calculation
- **Real-time Analytics** - ClickHouse-powered analytics for business insights

### Technical Features
- **Event-Driven Architecture** - Kafka-based async communication between services
- **API Gateway** - Single entry point with authentication middleware and routing
- **Redis Caching** - Menu caching (5-min TTL) and session management
- **Email Notifications** - Async email sending for OTP and order status updates
- **Clean Architecture** - Domain-driven design with clear separation of concerns
- **Type Safety** - Rust's compile-time guarantees for reliability

## 📦 Technology Stack

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **Language** | Rust 2021 | All microservices |
| **Web Framework** | Actix Web 4.4 | HTTP servers |
| **OLTP Database** | CockroachDB | Transactional data |
| **Event Streaming** | Apache Kafka | Async messaging |
| **Cache/Sessions** | Redis | Caching & sessions |
| **Analytics DB** | ClickHouse | OLAP queries |
| **Email** | Lettre + SMTP | Email delivery |
| **HTTP Client** | Reqwest | Inter-service calls |

## 📋 Prerequisites

- **Rust** 1.70+ with Cargo
- **Docker** & Docker Compose
- **Git**

## 🛠️ Quick Start

### 1. Clone the Repository
```bash
git clone <repository-url>
cd restman
```

### 2. Start Infrastructure Services
```bash
docker-compose up -d
```

This starts:
- CockroachDB (SQL: 26257, UI: 8080)
- Kafka + Zookeeper (9092, 2181)
- Redis (6379)
- ClickHouse (HTTP: 8124, Native: 9001)
- Kafka UI (8090)

### 3. Configure Environment Variables
Create a `.env` file in the project root:
```bash
# Database
DATABASE_URL=postgresql://root@localhost:26257/defaultdb?sslmode=disable

# Redis
REDIS_URL=redis://localhost:6379

# Kafka
KAFKA_BROKERS=localhost:9092

# ClickHouse
CLICKHOUSE_URL=http://localhost:8124

# Email (SMTP)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM=your-email@gmail.com

# Service URLs (for inter-service communication)
AUTH_SERVICE_URL=http://localhost:8001
RESTAURANT_SERVICE_URL=http://localhost:8002
ORDER_SERVICE_URL=http://localhost:8003
KITCHEN_SERVICE_URL=http://localhost:8004
BILLING_SERVICE_URL=http://localhost:8005
ANALYTICS_SERVICE_URL=http://localhost:8006
```

### 4. Run Database Migrations
```bash
# Auth Service
cd services/auth-service && sqlx migrate run && cd ../..

# Restaurant Service (use SKIP_MIGRATIONS if needed)
cd services/restaurant-service && sqlx migrate run && cd ../..

# Order Service
cd services/order-service && sqlx migrate run && cd ../..

# Kitchen Service
cd services/kitchen-service && sqlx migrate run && cd ../..

# Billing Service
cd services/billing-service && sqlx migrate run && cd ../..
```

### 5. Create Kafka Topics
```bash
docker exec -it restman-kafka kafka-topics --create --topic user.events --bootstrap-server localhost:9092 --partitions 3 --replication-factor 1
docker exec -it restman-kafka kafka-topics --create --topic menu.events --bootstrap-server localhost:9092 --partitions 3 --replication-factor 1
docker exec -it restman-kafka kafka-topics --create --topic order.events --bootstrap-server localhost:9092 --partitions 3 --replication-factor 1
docker exec -it restman-kafka kafka-topics --create --topic bill.events --bootstrap-server localhost:9092 --partitions 3 --replication-factor 1
```

### 6. Create ClickHouse Tables
```bash
curl -X POST 'http://localhost:8124/' -d "
CREATE DATABASE IF NOT EXISTS restman_analytics;

CREATE TABLE IF NOT EXISTS restman_analytics.orders (
    order_id String,
    user_id String,
    restaurant_id String,
    total_amount Decimal(10, 2),
    status String,
    created_at DateTime,
    updated_at DateTime,
    event_date Date
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(event_date)
ORDER BY (event_date, created_at);

CREATE TABLE IF NOT EXISTS restman_analytics.order_items (
    order_item_id String,
    order_id String,
    food_id String,
    food_name String,
    quantity Int32,
    unit_price Decimal(10, 2),
    total_price Decimal(10, 2),
    created_at DateTime,
    event_date Date
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(event_date)
ORDER BY (event_date, order_id);

CREATE TABLE IF NOT EXISTS restman_analytics.bills (
    bill_id String,
    order_id String,
    user_id String,
    restaurant_id String,
    subtotal Decimal(10, 2),
    tax Decimal(10, 2),
    total Decimal(10, 2),
    payment_status String,
    created_at DateTime,
    paid_at Nullable(DateTime),
    event_date Date
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(event_date)
ORDER BY (event_date, created_at);
"
```

### 7. Start All Services

Open 7 terminal windows and run each service:

```bash
# Terminal 1: API Gateway
cargo run -p gateway

# Terminal 2: Auth Service
cargo run -p auth-service

# Terminal 3: Restaurant Service
SKIP_MIGRATIONS=true cargo run -p restaurant-service

# Terminal 4: Order Service
cargo run -p order-service

# Terminal 5: Kitchen Service
cargo run -p kitchen-service

# Terminal 6: Billing Service
cargo run -p billing-service

# Terminal 7: Analytics Service
cargo run -p analytics-service
```

### 8. Verify All Services
```bash
curl http://localhost:8000/health                    # Gateway
curl http://localhost:8000/api/auth/health           # Auth
curl http://localhost:8000/api/restaurants/health    # Restaurant
curl http://localhost:8000/api/orders/health         # Order
curl http://localhost:8000/api/kitchen/health        # Kitchen
curl http://localhost:8000/api/billing/health        # Billing
curl http://localhost:8000/api/analytics/health      # Analytics
```

All should return `{"status":"healthy","service":"..."}`

## 📚 API Documentation

### Using Postman Collection

Import the Postman collection and environment:
1. **Collection**: `RestMan_Complete_API.postman_collection.json`
2. **Environment**: `RestMan_Complete.postman_environment.json`

The collection includes 37 endpoints across 6 services, all configured to use the API Gateway.

### Authentication Flow

1. **Request OTP**
```bash
curl -X POST http://localhost:8000/api/auth/request-otp \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com"}'
```

2. **Verify OTP** (check email for code)
```bash
curl -X POST http://localhost:8000/api/auth/verify-otp \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "otp": "123456"
  }'
```

Response includes `session_token` - use this in Authorization header for subsequent requests.

3. **Authenticated Requests**
```bash
curl -X GET http://localhost:8000/api/auth/me \
  -H "Authorization: Bearer <session_token>"
```

### Complete User Journey

1. **Authenticate** → Get session token
2. **Create Restaurant** → POST `/api/restaurants`
3. **Create Category** → POST `/api/categories`
4. **Add Food Items** → POST `/api/foods`
5. **Place Order** → POST `/api/orders`
6. **Kitchen Updates** → PATCH `/api/kitchen/tickets/{id}/status`
7. **View Bill** → GET `/api/billing/bills/order/{order_id}`
8. **Analytics** → GET `/api/analytics/top-foods`

## 🏛️ Project Structure

```
restman/
├── gateway/                    # API Gateway (Port 8000)
├── services/
│   ├── auth-service/          # Authentication (Port 8001)
│   ├── restaurant-service/    # Restaurant & Menu (Port 8002)
│   ├── order-service/         # Order Management (Port 8003)
│   ├── kitchen-service/       # Kitchen Operations (Port 8004)
│   ├── billing-service/       # Billing (Port 8005)
│   ├── analytics-service/     # Analytics (Port 8006)
│   └── scheduler-service/     # Background Jobs
├── shared/
│   ├── models/                # Shared domain models
│   ├── kafka-client/          # Kafka utilities
│   ├── db-utils/              # Database utilities
│   └── error-handling/        # Error types
├── docker-compose.yml         # Infrastructure services
├── Cargo.toml                 # Workspace configuration
└── README.md                  # This file
```

## 🔄 Event Flow

### Order Placement Flow
```
1. User places order → Order Service
2. Order Service validates food items → Restaurant Service (HTTP)
3. Order Service creates order → Database
4. Order Service publishes → Kafka: order.placed
5. Kitchen Service consumes → Creates kitchen ticket
6. Billing Service consumes → Generates bill
7. Analytics Service consumes → Stores in ClickHouse
```

### Kitchen Status Update Flow
```
1. Kitchen updates ticket status → Kitchen Service
2. Kitchen Service publishes → Kafka: order.status_updated
3. Order Service consumes → Updates order status
4. Kitchen Service sends email → User (when status = READY)
```

## 🧪 Testing

### Health Checks
```bash
# Check all services through gateway
for service in auth restaurants orders kitchen billing analytics; do
  echo "Testing $service..."
  curl -s http://localhost:8000/api/$service/health | jq
done
```

### Monitor Kafka Events
Access Kafka UI at http://localhost:8090 to view:
- Topics: `user.events`, `menu.events`, `order.events`, `bill.events`
- Consumer groups and lag
- Message contents

### View Analytics
```bash
# Top 10 foods by order count
curl http://localhost:8000/api/analytics/top-foods?limit=10

# Revenue by restaurant
curl http://localhost:8000/api/analytics/revenue?start_date=2024-01-01&end_date=2024-12-31

# Orders by status
curl http://localhost:8000/api/analytics/orders-by-status
```

## 🔧 Configuration

Each service can be configured via environment variables:

### Common Variables
- `DATABASE_URL` - CockroachDB connection string
- `REDIS_URL` - Redis connection string
- `KAFKA_BROKERS` - Kafka broker addresses
- `RUST_LOG` - Logging level (info, debug, trace)

### Service-Specific
- **Auth Service**: `SMTP_*` for email configuration
- **Gateway**: All service URLs for routing
- **Analytics**: `CLICKHOUSE_URL` for analytics database

## 📊 Monitoring

### CockroachDB Admin UI
- URL: http://localhost:8080
- View database metrics, queries, and cluster health

### Kafka UI
- URL: http://localhost:8090
- Monitor topics, consumer groups, and message flow

### Service Logs
Each service outputs structured JSON logs with tracing information.

## 🐛 Troubleshooting

### Restaurant Service Migration Error
If you see migration errors, start with:
```bash
SKIP_MIGRATIONS=true cargo run -p restaurant-service
```

### Kafka Topics Not Found
Create topics manually:
```bash
docker exec -it restman-kafka kafka-topics --create --topic <topic-name> \
  --bootstrap-server localhost:9092 --partitions 3 --replication-factor 1
```

### ClickHouse Connection Issues
Verify ClickHouse is running and accessible:
```bash
curl http://localhost:8124/ping
```

### Email Not Sending
Check SMTP credentials in `.env` and ensure:
- Using app-specific password for Gmail
- SMTP port 587 is not blocked

## 🚢 Deployment

### Production Considerations
1. **Database**: Use CockroachDB cluster (3+ nodes)
2. **Kafka**: Multi-broker setup with replication
3. **Redis**: Redis Cluster or Sentinel for HA
4. **Services**: Deploy behind load balancer
5. **Secrets**: Use secret management (Vault, AWS Secrets Manager)
6. **Monitoring**: Add Prometheus + Grafana
7. **Logging**: Centralized logging (ELK, Loki)

### Docker Build
```bash
# Build all services
cargo build --release

# Create Docker images (Dockerfile needed for each service)
docker build -t restman-gateway ./gateway
docker build -t restman-auth ./services/auth-service
# ... etc
```

## 📝 License

MIT License - See LICENSE file for details

## 👥 Authors

Restaurant Management Team

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test --workspace`
5. Submit a pull request

## 📞 Support

For issues and questions:
- Open an issue on GitHub
- Check existing documentation
- Review Postman collection for API examples

