-- Create foods table in restaurant schema
CREATE TABLE IF NOT EXISTS restaurant.foods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    restaurant_id UUID NOT NULL REFERENCES restaurant.restaurants(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES restaurant.food_categories(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price DECIMAL(10, 2) NOT NULL CHECK (price >= 0),
    is_available BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on restaurant_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_foods_restaurant ON restaurant.foods(restaurant_id);

-- Create index on category_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_foods_category ON restaurant.foods(category_id);

-- Create index on is_available for filtering
CREATE INDEX IF NOT EXISTS idx_foods_available ON restaurant.foods(is_available);

