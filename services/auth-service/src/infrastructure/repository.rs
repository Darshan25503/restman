use chrono::Utc;
use error_handling::AppResult;
use models::user::{User, UserRole};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, role, is_active, last_verified_at, created_at, updated_at
            FROM auth.users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(&self, user_id: Uuid) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, role, is_active, last_verified_at, created_at, updated_at
            FROM auth.users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Create a new user
    pub async fn create(&self, email: &str, role: UserRole) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO auth.users (id, email, role, is_active, last_verified_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, email, role, is_active, last_verified_at, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(email)
        .bind(role.to_string())
        .bind(true)
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("User created: {} with role: {:?}", email, role);
        Ok(user)
    }

    /// Update user's last_verified_at timestamp
    pub async fn update_last_verified(&self, user_id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE auth.users
            SET last_verified_at = $1, updated_at = $2
            WHERE id = $3
            "#,
        )
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Deactivate user
    pub async fn deactivate(&self, user_id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE auth.users
            SET is_active = false, updated_at = $1
            WHERE id = $2
            "#,
        )
        .bind(Utc::now())
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        tracing::info!("User deactivated: {}", user_id);
        Ok(())
    }

    /// Find users who haven't verified in the last 2 hours
    pub async fn find_inactive_users(&self, hours: i64) -> AppResult<Vec<User>> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(hours);
        
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, role, is_active, last_verified_at, created_at, updated_at
            FROM auth.users
            WHERE is_active = true AND last_verified_at < $1
            "#,
        )
        .bind(cutoff_time)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }
}

