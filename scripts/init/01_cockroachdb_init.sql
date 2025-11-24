-- Create main database
CREATE DATABASE IF NOT EXISTS restman_db;

-- Create schemas for each microservice
CREATE SCHEMA IF NOT EXISTS restman_db.auth;
CREATE SCHEMA IF NOT EXISTS restman_db.restaurant;
CREATE SCHEMA IF NOT EXISTS restman_db.orders;
CREATE SCHEMA IF NOT EXISTS restman_db.kitchen;
CREATE SCHEMA IF NOT EXISTS restman_db.billing;

-- Verify schemas
SHOW SCHEMAS FROM restman_db;

