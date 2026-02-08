use std::collections::hash_map::Entry;
use crate::domain::email::Email;
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
