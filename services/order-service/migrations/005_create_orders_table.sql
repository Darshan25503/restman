-- Create orders table
CREATE TABLE IF NOT EXISTS orders.orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    restaurant_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'PLACED',
    total_amount NUMERIC(10, 2) NOT NULL,
    delivery_address TEXT,
    special_instructions TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_status CHECK (status IN ('PLACED', 'ACCEPTED', 'IN_PROGRESS', 'READY', 'COMPLETED', 'CANCELLED')),
    CONSTRAINT positive_total CHECK (total_amount >= 0)
);

-- Create indexes
CREATE INDEX idx_orders_user_id ON orders.orders(user_id);
CREATE INDEX idx_orders_restaurant_id ON orders.orders(restaurant_id);
CREATE INDEX idx_orders_status ON orders.orders(status);
CREATE INDEX idx_orders_created_at ON orders.orders(created_at DESC);

-- Create updated_at trigger
CREATE OR REPLACE FUNCTION orders.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_orders_updated_at BEFORE UPDATE ON orders.orders
    FOR EACH ROW EXECUTE FUNCTION orders.update_updated_at_column();

