#!/bin/bash

set -e

echo "=========================================="
echo "Restaurant Management System - Setup"
echo "=========================================="
echo ""

# Step 1: Start all services
echo "Step 1: Starting all infrastructure services..."
docker-compose up -d

echo ""
echo "Waiting for services to be healthy..."
sleep 15

# Step 2: Initialize CockroachDB
echo ""
echo "Step 2: Initializing CockroachDB..."
docker exec -i restman-cockroachdb cockroach sql --insecure < scripts/init/01_cockroachdb_init.sql

# Step 3: Initialize ClickHouse
echo ""
echo "Step 3: Initializing ClickHouse..."
docker exec -i restman-clickhouse clickhouse-client --multiquery < scripts/init/02_clickhouse_init.sql

# Step 4: Create Kafka topics
echo ""
echo "Step 4: Creating Kafka topics..."
chmod +x scripts/init/03_kafka_topics.sh
./scripts/init/03_kafka_topics.sh

# Step 5: Verify all services
echo ""
echo "Step 5: Verifying all services..."
echo ""

echo "✓ CockroachDB Admin UI: http://localhost:8080"
echo "✓ Kafka UI: http://localhost:8090"
echo "✓ ClickHouse HTTP: http://localhost:8124"
echo "✓ Redis: localhost:6379"
echo ""

echo "=========================================="
echo "Setup Complete! 🚀"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. Check CockroachDB schemas: docker exec -it restman-cockroachdb cockroach sql --insecure -e 'SHOW SCHEMAS FROM restman_db;'"
echo "2. Check ClickHouse tables: docker exec -it restman-clickhouse clickhouse-client -q 'SHOW TABLES FROM restman_analytics;'"
echo "3. Check Kafka topics: docker exec restman-kafka kafka-topics --list --bootstrap-server localhost:9093"
echo "4. Test Redis: docker exec restman-redis redis-cli ping"
echo ""

