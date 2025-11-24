-- Create users table in auth schema
CREATE TABLE IF NOT EXISTS auth.users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    role TEXT NOT NULL CHECK (role IN ('user', 'rest', 'kitch')),
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_verified_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on email for faster lookups
CREATE INDEX IF NOT EXISTS idx_users_email ON auth.users(email);

-- Create index on is_active and last_verified_at for scheduler queries
CREATE INDEX IF NOT EXISTS idx_users_active_verified ON auth.users(is_active, last_verified_at);

