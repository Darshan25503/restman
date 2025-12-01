-- Create bills analytics table
CREATE TABLE IF NOT EXISTS restman_analytics.bills (
    id UUID,
    order_id UUID,
    user_id UUID,
    restaurant_id UUID,
    subtotal Decimal(10, 2),
    tax_amount Decimal(10, 2),
    discount_amount Decimal(10, 2),
    total_amount Decimal(10, 2),
    status String,
    payment_method String,
    generated_at DateTime,
    paid_at Nullable(DateTime),
    created_at DateTime,
    event_timestamp DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY (restaurant_id, created_at, id)
PARTITION BY toYYYYMM(created_at);

