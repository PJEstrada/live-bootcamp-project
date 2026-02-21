use sqlx::PgPool;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    email::Email,
    HashedPassword,
    user::User,
};

// Intermediate struct that matches the DB columns exactly.
// sqlx::query_as! maps rows directly into this.
struct PgUserRow {
    email: String,
    password_hash: String,
    requires_2fa: bool,
}

impl TryFrom<PgUserRow> for User {
    type Error = UserStoreError;

    fn try_from(row: PgUserRow) -> Result<Self, Self::Error> {
        let email = Email::parse(row.email)
            .map_err(|_| UserStoreError::UnexpectedError)?;
        let password = HashedPassword::parse_password_hash(row.password_hash)
            .map_err(|_| UserStoreError::UnexpectedError)?;
        Ok(User::new(email, password, row.requires_2fa))
    }
}

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        // sqlx::query! checks the SQL at compile time, and bind params are passed inline
        sqlx::query!(
            "INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)",
            user.email.as_ref(),
            user.password.as_ref(),
            user.requires_2fa
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            // unique constraint violation = user already exists
            match e {
                sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                    UserStoreError::UserAlreadyExists
                }
                _ => UserStoreError::UnexpectedError,
            }
        })?;

        Ok(())
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        // sqlx::query_as! checks the SQL at compile time and maps columns
        // directly into PgUserRow fields by name.
        sqlx::query_as!(
            PgUserRow,
            "SELECT email, password_hash, requires_2fa FROM users WHERE email = $1",
            email.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?
        .ok_or(UserStoreError::UserNotFound)?
        .try_into()
    }

    async fn validate_user(&self, email: &Email, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;

        user.password
            .verify_raw_password(password)
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)?;

        Ok(())
    }
    // TODO: Implement all required methods. Note that you will need to make SQL queries against our PostgreSQL instance inside these methods. Ensure to parse the password_hash.
}