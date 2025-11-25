-- Create food_categories table in restaurant schema
CREATE TABLE IF NOT EXISTS restaurant.food_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    restaurant_id UUID NOT NULL REFERENCES restaurant.restaurants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    display_order INT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on restaurant_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_food_categories_restaurant ON restaurant.food_categories(restaurant_id);

-- Create index on is_active for filtering
CREATE INDEX IF NOT EXISTS idx_food_categories_active ON restaurant.food_categories(is_active);

-- Create index on display_order for sorting
CREATE INDEX IF NOT EXISTS idx_food_categories_order ON restaurant.food_categories(restaurant_id, display_order);

