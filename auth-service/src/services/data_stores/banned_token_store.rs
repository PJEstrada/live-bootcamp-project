use std::collections::HashSet;
use crate::domain::data_stores::{BannedTokenStore, BannedTokenStoreError, UserStoreError};

#[derive(Default, Debug, Clone)]
pub struct HashsetBannedTokenStore {
    tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        if self.tokens.contains(&token) {
            return Err(BannedTokenStoreError::TokenAlreadyBanned)
        }
        self.tokens.insert(token);
        Ok(())
    }

    async fn is_token_banned(&self, token: String) -> bool {
        self.tokens.contains(&token)
    }
}


impl HashsetBannedTokenStore {
    pub fn new() -> Self {
        Self { tokens: HashSet::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;


    #[tokio::test]
    pub async fn test_add_token() {
        let mut store = HashsetBannedTokenStore::new();
        store.add_token("token".to_string()).await.unwrap();
        assert!(store.is_token_banned("token".to_string()).await);
    }

    #[tokio::test]
    pub async fn test_is_token_banned() {
        let mut store = HashsetBannedTokenStore::new();
        assert!(!store.is_token_banned("token".to_string()).await);

        // add token to store
        store.add_token("token2".to_string()).await.unwrap();
        assert!(store.is_token_banned("token2".to_string()).await);
    }

}