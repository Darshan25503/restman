#!/bin/bash

echo "=========================================="
echo "Stopping all services..."
echo "=========================================="

docker-compose down

echo ""
echo "Services stopped."
echo ""
echo "To remove all data volumes, run:"
echo "  docker-compose down -v"
echo ""

