use std::collections::HashMap;
use std::collections::hash_map::Entry;
use crate::domain::data_stores::{UserStore, UserStoreError};
use crate::domain::email::Email;
use crate::domain::Password;
use crate::domain::user::User;

#[derive(Default, Debug, Clone)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        match self.users.entry(user.email.clone()) {
            Entry::Occupied(_) => Err(UserStoreError::UserAlreadyExists),
            Entry::Vacant(entry) => {
                entry.insert(user);
                Ok(())
            }
        }

    }

     async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

     async  fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        if user.password == password.clone() {
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
        assert_eq!(store.users.get(&user.email), Some(&user));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut store = HashmapUserStore::default();
        let user1 = User::new(Email("usr1@mail.com".to_string()),Password( "password".to_string()), false);
        let user2 = User::new(Email("usr2@mail.com".to_string()), Password("password".to_string()), false);
        store.add_user(user1.clone()).await.unwrap();
        store.add_user(user2.clone()).await.unwrap();
        let user_from_store1 = store.get_user(&user1.email).await.unwrap().clone();
        let user_from_store2 = store.get_user(&user2.email).await.unwrap().clone();
        assert_eq!(user_from_store1, user1);
        assert_eq!(user_from_store2, user2);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new(Email("usr1@mail.com".to_string()), Password("password".to_string()), false);
        store.add_user(user.clone()).await.unwrap();
        assert!(store.validate_user(&user.email, &user.password).await.is_ok());
        assert!(store.validate_user(&user.email, &Password("wrong_password".to_string())).await.is_err());
    }
}