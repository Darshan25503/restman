# 🏗️ Restaurant Management System - Workspace Structure

This document provides a visual overview of the complete workspace structure.

---

## 📁 Directory Tree

```
restman/
│
├── 📄 Cargo.toml                          # Root workspace configuration
├── 📄 .env                                # Environment variables (gitignored)
├── 📄 .env.example                        # Environment template
├── 📄 .gitignore                          # Git ignore rules
│
├── 📄 SYSTEM_DESIGN.md                    # Complete system design document
├── 📄 INFRASTRUCTURE.md                   # Infrastructure setup guide
├── 📄 PHASE_0_COMPLETE.md                 # Phase 0 completion summary
├── 📄 PHASE_1_COMPLETE.md                 # Phase 1 completion summary
├── 📄 WORKSPACE_STRUCTURE.md              # This file
│
├── 🐳 docker-compose.yml                  # Infrastructure services
│
├── 📂 scripts/                            # Setup and utility scripts
│   ├── setup.sh                           # One-command infrastructure setup
│   ├── teardown.sh                        # Cleanup script
│   ├── verify.sh                          # Verify all services
│   └── init/                              # Initialization scripts
│       ├── 01_cockroachdb_init.sql        # CockroachDB schemas
│       ├── 02_clickhouse_init.sql         # ClickHouse tables
│       └── 03_kafka_topics.sh             # Kafka topic creation
│
├── 📂 shared/                             # Shared libraries (workspace members)
│   │
│   ├── 📦 models/                         # Domain models & DTOs
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                     # Module exports
│   │       ├── user.rs                    # User, OTP, Session models
│   │       ├── restaurant.rs              # Restaurant, Category, Food models
│   │       ├── order.rs                   # Order, OrderItem models
│   │       ├── kitchen.rs                 # KitchenTicket models
│   │       ├── billing.rs                 # Bill models
│   │       └── events.rs                  # Kafka event schemas
│   │
│   ├── 📦 error-handling/                 # Common error types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs                     # AppError, AppResult
│   │
│   ├── 📦 kafka-client/                   # Kafka utilities
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                     # Core functions
│   │       ├── producer.rs                # KafkaProducer wrapper
│   │       └── consumer.rs                # KafkaConsumer wrapper
│   │
│   └── 📦 db-utils/                       # Database utilities
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs                     # Pool creation, health checks
│
├── 📂 services/                           # Microservices (workspace members)
│   │
│   ├── 🔐 auth-service/                   # Authentication & OTP
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs                    # (Placeholder)
│   │
│   ├── 🍽️ restaurant-service/             # Restaurant & menu management
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs                    # (Placeholder)
│   │
│   ├── 📋 order-service/                  # Order management
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs                    # (Placeholder)
│   │
│   ├── 👨‍🍳 kitchen-service/                 # Kitchen ticket tracking
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs                    # (Placeholder)
│   │
│   ├── 💰 billing-service/                # Bill generation & payment
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs                    # (Placeholder)
│   │
│   ├── 📊 analytics-service/              # ClickHouse analytics
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs                    # (Placeholder)
│   │
│   └── ⏰ scheduler-service/               # Cron jobs (user deactivation)
│       ├── Cargo.toml
│       └── src/
│           └── main.rs                    # (Placeholder)
│
├── 📂 gateway/                            # API Gateway (workspace member)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                        # (Placeholder)
│
└── 📂 target/                             # Build artifacts (gitignored)
```

---

## 📦 Workspace Members

| Member | Type | Purpose | Status |
|--------|------|---------|--------|
| `shared/models` | Library | Domain models & DTOs | ✅ Complete |
| `shared/error-handling` | Library | Error types & handling | ✅ Complete |
| `shared/kafka-client` | Library | Kafka producer/consumer | ✅ Complete |
| `shared/db-utils` | Library | Database utilities | ✅ Complete |
| `services/auth-service` | Binary | Authentication & OTP | 🔲 Placeholder |
| `services/restaurant-service` | Binary | Restaurant & menu | 🔲 Placeholder |
| `services/order-service` | Binary | Order management | 🔲 Placeholder |
| `services/kitchen-service` | Binary | Kitchen tickets | 🔲 Placeholder |
| `services/billing-service` | Binary | Billing & payment | 🔲 Placeholder |
| `services/analytics-service` | Binary | Analytics queries | 🔲 Placeholder |
| `services/scheduler-service` | Binary | Cron jobs | 🔲 Placeholder |
| `gateway` | Binary | API Gateway | 🔲 Placeholder |

---

## 🔗 Dependency Graph

```
┌─────────────────────────────────────────────────────────────┐
│                     Workspace Dependencies                   │
│  (actix-web, tokio, sqlx, redis, rdkafka, serde, etc.)      │
└─────────────────────────────────────────────────────────────┘
                              ▲
                              │
        ┌─────────────────────┴─────────────────────┐
        │                                           │
┌───────┴────────┐                         ┌────────┴────────┐
│ Shared Libs    │                         │   Services      │
│                │                         │                 │
│ • models       │◄────────────────────────┤ • auth          │
│ • error-       │                         │ • restaurant    │
│   handling     │                         │ • order         │
│ • kafka-client │                         │ • kitchen       │
│ • db-utils     │                         │ • billing       │
│                │                         │ • analytics     │
│                │                         │ • scheduler     │
│                │                         │                 │
│                │◄────────────────────────┤ • gateway       │
└────────────────┘                         └─────────────────┘
```

---

## 🎯 Current Status

### ✅ Completed
- [x] Phase 0: Infrastructure Setup (CockroachDB, Kafka, Redis, ClickHouse)
- [x] Phase 1: Workspace & Shared Libraries
  - [x] Workspace structure
  - [x] All shared libraries (models, error-handling, kafka-client, db-utils)
  - [x] Service placeholders
  - [x] Environment configuration
  - [x] Successful compilation

### 🔲 Next Steps
- [ ] Phase 2: Auth Service implementation
- [ ] Phase 3: Restaurant Service implementation
- [ ] Phase 4: Order Service implementation
- [ ] Phase 5: Kitchen Service implementation
- [ ] Phase 6: Billing Service implementation
- [ ] Phase 7: Analytics Service implementation
- [ ] Phase 8: Scheduler Service implementation
- [ ] Phase 9: API Gateway implementation
- [ ] Phase 10: Integration & Testing

---

**Last Updated**: Phase 1 Complete
**Workspace Compiles**: ✅ Yes
**Total Crates**: 12 (4 libs + 7 services + 1 gateway)

