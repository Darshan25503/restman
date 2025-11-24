-- Create analytics database
CREATE DATABASE IF NOT EXISTS restman_analytics;

-- Create orders_analytics table
CREATE TABLE IF NOT EXISTS restman_analytics.orders_analytics (
    event_date Date,
    event_timestamp DateTime,
    order_id String,
    user_id String,
    restaurant_id String,
    order_type String,
    status String,
    total_price Float64,
    placed_at DateTime,
    completed_at Nullable(DateTime),
    payment_status String,
    payment_method Nullable(String)
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(event_date)
ORDER BY (event_date, restaurant_id, order_id);

-- Create order_items_analytics table
CREATE TABLE IF NOT EXISTS restman_analytics.order_items_analytics (
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

-- Verify tables
SHOW TABLES FROM restman_analytics;

