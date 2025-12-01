-- Create order items analytics table
CREATE TABLE IF NOT EXISTS restman_analytics.order_items (
    id UUID,
    order_id UUID,
    food_id UUID,
    food_name String,
    food_description String,
    quantity Int32,
    unit_price Decimal(10, 2),
    subtotal Decimal(10, 2),
    created_at DateTime,
    event_timestamp DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY (order_id, food_id, id)
PARTITION BY toYYYYMM(created_at);

