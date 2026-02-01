use std::collections::HashMap;
use std::collections::hash_map::Entry;
use crate::domain::data_stores::{UserStore, UserStoreError};
use crate::domain::email::Email;
use crate::domain::Password;
use crate::domain::user::User;


// TODO: Create a new struct called `HashmapUserStore` containing a `users` field
// which stores a `HashMap`` of email `String`s mapped to `User` objects.
// Derive the `Default` trait for `HashmapUserStore`.

#[derive(Default, Debug, Clone)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        match self.users.entry(user.email.as_ref().to_string()) {
            Entry::Occupied(_) => Err(UserStoreError::UserAlreadyExists),
            Entry::Vacant(entry) => {
                entry.insert(user);
                Ok(())
            }
        }

    }

     async fn get_user(&mut self, email: Email) -> Result<User, UserStoreError> {
        match self.users.entry(email.as_ref().to_string()) {
            Entry::Occupied(entry) => {
                Ok(entry.get().clone())
            },
            Entry::Vacant(entry) => Err(UserStoreError::UserNotFound)
        }
    }

     async  fn validate_user(&mut self, email: Email, password: Password) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        if user.password == password {
            Ok(())
        } else {
            Err(UserStoreError::InvalidCredentials)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashmapUserStore::default();
        let mail = Email("test@test.com".to_string());
        let pass = Password("password@test.com".to_string());
        let user = User::new(mail, pass, false);
        store.add_user(user.clone()).await.unwrap();
        assert_eq!(store.users.get(&user.email.as_ref().to_string()), Some(&user));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut store = HashmapUserStore::default();
        let user1 = User::new(Email("usr1@mail.com".to_string()),Password( "password".to_string()), false);
        let user2 = User::new(Email("usr2@mail.com".to_string()), Password("password".to_string()), false);
        store.add_user(user1.clone()).await.unwrap();
        store.add_user(user2.clone()).await.unwrap();
        let user_from_store1 = store.get_user(user1.email.clone()).await.unwrap().clone();
        let user_from_store2 = store.get_user(user2.email.clone()).await.unwrap().clone();
        assert_eq!(user_from_store1, user1);
        assert_eq!(user_from_store2, user2);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new(Email("usr1@mail.com".to_string()), Password("password".to_string()), false);
        store.add_user(user.clone()).await.unwrap();
        assert!(store.validate_user(user.email.clone(), user.password).await.is_ok());
        assert!(store.validate_user(user.email, Password("wrong_password".to_string())).await.is_err());
    }
}