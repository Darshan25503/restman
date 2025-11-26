-- Create bills table
CREATE TABLE billing.bills (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL UNIQUE,
    user_id UUID NOT NULL,
    restaurant_id UUID NOT NULL,
    subtotal DECIMAL(10, 2) NOT NULL,
    tax_amount DECIMAL(10, 2) DEFAULT 0,
    discount_amount DECIMAL(10, 2) DEFAULT 0,
    total_amount DECIMAL(10, 2) NOT NULL,
    status VARCHAR(20) DEFAULT 'PENDING',
    payment_method VARCHAR(20),
    generated_at TIMESTAMPTZ DEFAULT now(),
    paid_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Create index on order_id for fast lookups
CREATE INDEX idx_bills_order_id ON billing.bills(order_id);

-- Create index on user_id for user bill history
CREATE INDEX idx_bills_user_id ON billing.bills(user_id);

-- Create index on restaurant_id for restaurant revenue tracking
CREATE INDEX idx_bills_restaurant_id ON billing.bills(restaurant_id);

-- Create index on status for filtering
CREATE INDEX idx_bills_status ON billing.bills(status);

