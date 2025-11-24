#!/bin/bash

echo "=========================================="
echo "Verifying Infrastructure Setup"
echo "=========================================="
echo ""

# Check CockroachDB
echo "1. CockroachDB Schemas:"
docker exec restman-cockroachdb cockroach sql --insecure -e "SHOW SCHEMAS FROM restman_db;" 2>/dev/null
if [ $? -eq 0 ]; then
    echo "   ✓ CockroachDB is running"
else
    echo "   ✗ CockroachDB is not accessible"
fi
echo ""

# Check ClickHouse
echo "2. ClickHouse Tables:"
docker exec restman-clickhouse clickhouse-client -q "SHOW TABLES FROM restman_analytics;" 2>/dev/null
if [ $? -eq 0 ]; then
    echo "   ✓ ClickHouse is running"
else
    echo "   ✗ ClickHouse is not accessible"
fi
echo ""

# Check Kafka
echo "3. Kafka Topics:"
docker exec restman-kafka kafka-topics --list --bootstrap-server localhost:9093 2>/dev/null
if [ $? -eq 0 ]; then
    echo "   ✓ Kafka is running"
else
    echo "   ✗ Kafka is not accessible"
fi
echo ""

# Check Redis
echo "4. Redis:"
docker exec restman-redis redis-cli ping 2>/dev/null
if [ $? -eq 0 ]; then
    echo "   ✓ Redis is running"
else
    echo "   ✗ Redis is not accessible"
fi
echo ""

echo "=========================================="
echo "Verification Complete"
echo "=========================================="
echo ""
echo "Access Points:"
echo "  - CockroachDB Admin: http://localhost:8080"
echo "  - Kafka UI: http://localhost:8090"
echo "  - ClickHouse HTTP: http://localhost:8124"
echo ""

