#!/bin/bash

# Wait for Kafka to be ready
echo "Waiting for Kafka to be ready..."
sleep 10

# Create Kafka topics
echo "Creating Kafka topics..."

docker exec restman-kafka kafka-topics --create \
  --bootstrap-server localhost:9093 \
  --topic user.events \
  --partitions 3 \
  --replication-factor 1 \
  --if-not-exists

docker exec restman-kafka kafka-topics --create \
  --bootstrap-server localhost:9093 \
  --topic menu.events \
  --partitions 3 \
  --replication-factor 1 \
  --if-not-exists

docker exec restman-kafka kafka-topics --create \
  --bootstrap-server localhost:9093 \
  --topic order.events \
  --partitions 3 \
  --replication-factor 1 \
  --if-not-exists

docker exec restman-kafka kafka-topics --create \
  --bootstrap-server localhost:9093 \
  --topic bill.events \
  --partitions 3 \
  --replication-factor 1 \
  --if-not-exists

# List all topics
echo "Listing all topics..."
docker exec restman-kafka kafka-topics --list --bootstrap-server localhost:9093

echo "Kafka topics created successfully!"

