use std::collections::hash_map::Entry;
use rand::Rng;
use serde::Deserialize;
use crate::domain::email::Email;
use crate::domain::error::TwoFaError;
use crate::domain::Password;
use crate::domain::user::User;

#[async_trait::async_trait]
pub trait BannedTokenStore {

    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError>;
    async fn is_token_banned(&self, token: String) -> bool;

}
#[async_trait::async_trait]
pub trait UserStore {

     async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;

     async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> ;

     async  fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError> ;
}

// This trait represents the interface all concrete 2FA code stores should implement
#[async_trait::async_trait]
pub trait TwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFaCode,
    ) -> Result<(), TwoFACodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFaCode), TwoFACodeStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}


#[derive(Debug, PartialEq)]
pub enum BannedTokenStoreError {
    TokenAlreadyBanned,
}



#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self, String> {
        if uuid::Uuid::parse_str(&id).is_err() {
            Err(format!("{} is not a valid uuid", id))?
        }
        Ok(LoginAttemptId(id))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId(uuid::Uuid::new_v4().to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct TwoFaCode(String);

impl TwoFaCode {

    fn is_digits(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_numeric())
    }
    pub fn parse(code: String) -> Result<Self, anyhow::Error> {
        // Ensure `code` is a valid 6-digit code
        if code.len() != 6 {
            return Err(anyhow::Error::from(TwoFaError::InvalidCode));
        }

        if !Self::is_digits(code.as_str()) {
            return Err(anyhow::Error::from(TwoFaError::InvalidCode));
        }
        Ok(TwoFaCode(code))
    }
}

impl Default for TwoFaCode {
    fn default() -> Self {
        // Use the `rand` crate to generate a random 2FA code.
        // The code should be 6 digits (ex: 834629)
        let code: u32 = rand::rng().random_range(100_000..1_000_000);
        TwoFaCode(code.to_string())
    }
}

impl AsRef<str> for TwoFaCode {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}