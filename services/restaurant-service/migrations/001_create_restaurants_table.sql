-- Create restaurants table in restaurant schema
CREATE TABLE IF NOT EXISTS restaurant.restaurants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    address TEXT,
    phone VARCHAR(20),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on owner_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_restaurants_owner ON restaurant.restaurants(owner_id);

-- Create index on is_active for filtering
CREATE INDEX IF NOT EXISTS idx_restaurants_active ON restaurant.restaurants(is_active);

