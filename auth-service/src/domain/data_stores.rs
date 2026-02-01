use std::collections::hash_map::Entry;
use crate::domain::email::Email;
use crate::domain::Password;
use crate::domain::user::User;

#[async_trait::async_trait]
pub trait UserStore {

     async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;

     async fn get_user(&mut self, email: Email) -> Result<User, UserStoreError> ;

     async  fn validate_user(&mut self, email: Email, password: Password) -> Result<(), UserStoreError> ;
}

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}
