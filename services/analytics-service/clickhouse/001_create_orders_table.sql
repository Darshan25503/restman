-- Create orders analytics table
CREATE TABLE IF NOT EXISTS restman_analytics.orders (
    id UUID,
    user_id UUID,
    restaurant_id UUID,
    status String,
    total_amount Decimal(10, 2),
    delivery_address String,
    special_instructions String,
    created_at DateTime,
    updated_at DateTime,
    event_timestamp DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY (restaurant_id, created_at, id)
PARTITION BY toYYYYMM(created_at);

