use std::collections::HashMap;
use std::collections::hash_map::Entry;
use crate::domain::{
    data_stores::{LoginAttemptId, TwoFaCode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[derive(Default, Debug, Clone)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFaCode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFaCode,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        match self.codes.entry(email.clone()) {
            Entry::Occupied(mut entry) => {
                entry.remove();
                Ok(())
            }
            Entry::Vacant(_) => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }

    async fn get_code(&self, email: &Email) -> Result<(LoginAttemptId, TwoFaCode), TwoFACodeStoreError> {
        let result = self.codes.get(email);
        match result {
            Some(result) => Ok(result.clone()),
            None => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::domain::Password;
    use crate::domain::user::User;
    use crate::services::hashmap_user_store::HashmapUserStore;
    use super::*;

    #[tokio::test]
    async fn test_add_code() {
        let mut store = HashmapTwoFACodeStore::default();
        let mail = Email("test@test.com".to_string());
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFaCode::default();
        let add_res = store.add_code(mail.clone(), login_attempt_id.clone(), code.clone()).await;
        assert_eq!(add_res, Ok(()));
        let result = store.get_code(&mail).await.unwrap();
        assert_eq!(result.0, login_attempt_id);
        assert_eq!(result.1, code);
    }

    #[tokio::test]
    async fn test_remove_code() {
        let mut store = HashmapTwoFACodeStore::default();
        let mail = Email("test@test.com".to_string());
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFaCode::default();
        let add_res = store.add_code(mail.clone(), login_attempt_id.clone(), code.clone()).await;
        let remove_res = store.remove_code(&mail).await;
        assert_eq!(remove_res, Ok(()));

        // Not found
        let another_mail = Email("test2@test.com".to_string());
        let remove_err = store.remove_code(&another_mail).await;
        assert_eq!(remove_err, Err(TwoFACodeStoreError::LoginAttemptIdNotFound));
    }

    #[tokio::test]
    async fn test_get_code() {
        let mut store = HashmapTwoFACodeStore::default();
        let mail = Email("test@test.com".to_string());
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFaCode::default();
        let add_res = store.add_code(mail.clone(), login_attempt_id.clone(), code.clone()).await;
        let get_res = store.get_code(&mail).await;
        assert_eq!(get_res, Ok((login_attempt_id, code)));

        // Not found
        let another_mail = Email("test2@test.com".to_string());
        let get_err = store.remove_code(&another_mail).await;
        assert_eq!(get_err, Err(TwoFACodeStoreError::LoginAttemptIdNotFound));
    }

}
