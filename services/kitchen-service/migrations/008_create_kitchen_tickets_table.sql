-- Create kitchen_tickets table
CREATE TABLE IF NOT EXISTS kitchen.kitchen_tickets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL,
    restaurant_id UUID NOT NULL,
    user_id UUID NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'NEW',
    items JSONB NOT NULL,
    special_instructions TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_status CHECK (status IN ('NEW', 'ACCEPTED', 'IN_PROGRESS', 'READY', 'DELIVERED_TO_SERVICE'))
);

-- Create trigger for auto-updating updated_at
CREATE OR REPLACE FUNCTION kitchen.update_kitchen_tickets_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_kitchen_tickets_updated_at
    BEFORE UPDATE ON kitchen.kitchen_tickets
    FOR EACH ROW
    EXECUTE FUNCTION kitchen.update_kitchen_tickets_updated_at();

-- Create indexes
CREATE INDEX idx_kitchen_tickets_order_id ON kitchen.kitchen_tickets(order_id);
CREATE INDEX idx_kitchen_tickets_restaurant_id ON kitchen.kitchen_tickets(restaurant_id);
CREATE INDEX idx_kitchen_tickets_status ON kitchen.kitchen_tickets(status);
CREATE INDEX idx_kitchen_tickets_created_at ON kitchen.kitchen_tickets(created_at);

