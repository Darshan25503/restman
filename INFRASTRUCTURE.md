# Infrastructure Setup Guide

This guide covers Phase 0 of the Restaurant Management System - setting up all infrastructure components.

## Prerequisites

- Docker (version 20.10+)
- Docker Compose (version 2.0+)
- At least 4GB of free RAM
- Ports available: 6379, 8080, 8090, 8124, 9001, 9092, 26257

## Quick Start

### 1. Start Everything

```bash
chmod +x scripts/setup.sh
./scripts/setup.sh
```

This will:
- Start all Docker containers (CockroachDB, Kafka, Redis, ClickHouse)
- Initialize CockroachDB with schemas
- Initialize ClickHouse with analytics tables
- Create Kafka topics
- Verify all services

### 2. Verify Setup

```bash
chmod +x scripts/verify.sh
./scripts/verify.sh
```

### 3. Stop Everything

```bash
chmod +x scripts/teardown.sh
./scripts/teardown.sh
```

To remove all data:
```bash
docker-compose down -v
```

---

## Infrastructure Components

### 1. CockroachDB (OLTP Database)

**Ports:**
- SQL: `26257`
- Admin UI: `8080`

**Connection String:**
```
postgresql://root@localhost:26257/restman_db?sslmode=disable
```

**Schemas:**
- `auth` - User authentication and OTP
- `restaurant` - Restaurant and menu data
- `orders` - Order management
- `kitchen` - Kitchen tickets
- `billing` - Bills and payments

**Access Admin UI:**
```
http://localhost:8080
```

**Manual Commands:**
```bash
# Connect to SQL shell
docker exec -it restman-cockroachdb cockroach sql --insecure

# Show databases
docker exec restman-cockroachdb cockroach sql --insecure -e "SHOW DATABASES;"

# Show schemas
docker exec restman-cockroachdb cockroach sql --insecure -e "SHOW SCHEMAS FROM restman_db;"
```

---

### 2. Apache Kafka (Event Streaming)

**Ports:**
- External: `9092`
- Internal: `9093`

**Bootstrap Server:**
```
localhost:9092
```

**Topics:**
- `user.events` - User login, logout, deactivation
- `menu.events` - Menu CRUD operations
- `order.events` - Order lifecycle events
- `bill.events` - Billing events

**Access Kafka UI:**
```
http://localhost:8090
```

**Manual Commands:**
```bash
# List topics
docker exec restman-kafka kafka-topics --list --bootstrap-server localhost:9093

# Describe a topic
docker exec restman-kafka kafka-topics --describe --topic order.events --bootstrap-server localhost:9093

# Consume messages (from beginning)
docker exec restman-kafka kafka-console-consumer \
  --bootstrap-server localhost:9093 \
  --topic order.events \
  --from-beginning

# Produce a test message
docker exec -it restman-kafka kafka-console-producer \
  --bootstrap-server localhost:9093 \
  --topic user.events
```

---

### 3. Redis (Cache & Session Store)

**Port:** `6379`

**Connection String:**
```
redis://localhost:6379
```

**Usage:**
- OTP storage (TTL: 5 minutes)
- Session management (TTL: 2 hours)
- Menu caching (TTL: 5-10 minutes)

**Manual Commands:**
```bash
# Connect to Redis CLI
docker exec -it restman-redis redis-cli

# Test connection
docker exec restman-redis redis-cli ping

# Get all keys
docker exec restman-redis redis-cli KEYS '*'

# Monitor commands in real-time
docker exec restman-redis redis-cli MONITOR
```

---

### 4. ClickHouse (Analytics Database)

**Ports:**
- HTTP: `8124`
- Native: `9001`

**Connection String:**
```
http://localhost:8124
```

**Database:** `restman_analytics`

**Tables:**
- `orders_analytics` - Order-level analytics
- `order_items_analytics` - Item-level analytics

**Manual Commands:**
```bash
# Connect to ClickHouse client
docker exec -it restman-clickhouse clickhouse-client

# Show databases
docker exec restman-clickhouse clickhouse-client -q "SHOW DATABASES;"

# Show tables
docker exec restman-clickhouse clickhouse-client -q "SHOW TABLES FROM restman_analytics;"

# Query data
docker exec restman-clickhouse clickhouse-client -q "SELECT * FROM restman_analytics.orders_analytics LIMIT 10;"

# HTTP query
curl 'http://localhost:8124/?query=SELECT%20*%20FROM%20restman_analytics.orders_analytics%20LIMIT%2010'
```

---

## Troubleshooting

### Services won't start
```bash
# Check Docker is running
docker ps

# Check logs
docker-compose logs -f

# Restart specific service
docker-compose restart cockroachdb
```

### Port conflicts
```bash
# Check what's using a port
lsof -i :26257
# or
netstat -tuln | grep 26257

# Kill the process or change ports in docker-compose.yml
```

### Reset everything
```bash
# Stop and remove all containers and volumes
docker-compose down -v

# Remove all images (optional)
docker-compose down --rmi all

# Start fresh
./scripts/setup.sh
```

---

## Next Steps

Once infrastructure is running:

1. ✅ Verify all services are healthy
2. ✅ Check CockroachDB Admin UI (http://localhost:8080)
3. ✅ Check Kafka UI (http://localhost:8090)
4. ✅ Test Redis connection
5. ✅ Test ClickHouse connection
6. 🚀 Move to **Phase 1: Workspace Setup**

---

## Health Checks

All services have health checks configured. Check status:

```bash
docker-compose ps
```

All services should show `healthy` status.

---

**Phase 0 Complete!** 🎉

