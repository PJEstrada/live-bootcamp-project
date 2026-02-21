use std::error::Error;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct HashedPassword(pub String);

impl HashedPassword {

    // TODO:
    // Update the parse function. Note that it's now async.
    // After password validation, hash the password.
    // Using the provided helper function compute_password_hash.
    pub async fn parse(s: String) -> Result<Self, String> {
        if validate_password(&s) {
            let hash = Self::compute_password_hash(&s).await.map_err(|e| e.to_string())?;
            Ok(Self(hash))
        } else {
            Err("Failed to parse string to a Password type".to_owned())
        }

    }

    // Add a parse_password_hash function.
    // To validate the format of the hash string,
    // use PasswordHash::new
    pub fn parse_password_hash(
        hash: String
    ) -> Result<HashedPassword, String>{
        match PasswordHash::new(&hash) {
            Ok(_) => {
                Ok(HashedPassword(hash))
            }
            Err(_) => {
                Err("Failed to parse password hash".to_owned())
            }
        }
    }

    // Add a verify_raw_password function.
    // To verify the password candidate use
    // Argon2::default().verify_password.
    pub async fn verify_raw_password(
        &self,
        password_candidate: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let password_hash_str = self.as_ref().to_owned();
        let password_candidate = password_candidate.to_owned();


        // Do the CPU-heavy verification on the blocking thread pool.
        tokio::task::spawn_blocking(move || -> Result<(), Box<dyn Error + Send + Sync>> {
            let expected_password_hash = PasswordHash::new(&password_hash_str)?;
            Argon2::default().verify_password(
                password_candidate.as_bytes(),
                &expected_password_hash,
            )?;
            Ok(())
        })
            .await
            .map_err(|e| -> Box<dyn Error + Send + Sync> { e.into() })?
    }

    // Helper function to hash passwords before persisting them in storage.
    async fn compute_password_hash(password: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let password = password.to_owned();

        tokio::task::spawn_blocking(move || -> Result<String, Box<dyn Error + Send + Sync>> {
            let salt: SaltString = SaltString::generate(&mut OsRng);
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
                .hash_password(password.as_bytes(), &salt)?
                .to_string();

            Ok(password_hash)
        }).await.map_err(|e| -> Box<dyn Error + Send + Sync> { e.into() })?
    }
}

impl AsRef<str> for HashedPassword {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

fn validate_password(s: &str) -> bool {
    s.len() >= 8
}

#[cfg(test)]
mod tests {
    use super::HashedPassword; // updated!
    use argon2::{ // new
                  password_hash::{rand_core::OsRng, SaltString},
                  Algorithm, Argon2, Params, PasswordHasher, Version
    };
    use fake::faker::internet::en::Password as FakePassword;
    use fake::Fake;
    use quickcheck::Gen;
    use rand::SeedableRng;

    // updated!
    #[tokio::test]
    async fn empty_string_is_rejected() {
        let password = "".to_owned();

        // updated!
        assert!(HashedPassword::parse(password).await.is_err());
    }

    // updated!
    #[tokio::test]
    async fn string_less_than_8_characters_is_rejected() {
        let password = "1234567".to_owned();
        // updated!
        assert!(HashedPassword::parse(password).await.is_err());
    }

    // new
    #[test]
    fn can_parse_valid_argon2_hash() {
        // Arrange - Create a valid Argon2 hash
        let raw_password = "TestPassword123";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        );

        let hash_string = argon2
            .hash_password(raw_password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        // Act
        let hash_password = HashedPassword::parse_password_hash
            (hash_string.clone()).unwrap();

        // Assert
        assert_eq!(hash_password.as_ref(), hash_string.as_str());
        assert!(hash_password.as_ref().starts_with("$argon2id$v=19$"));
    }

    // new
    #[tokio::test]
    async fn can_verify_raw_password() {
        let raw_password = "TestPassword123";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        );

        let hash_string = argon2
            .hash_password(raw_password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        let hash_password = HashedPassword::parse_password_hash(
            hash_string
                .clone())
            .unwrap();

        assert_eq!(hash_password.as_ref(), hash_string.as_str());
        assert!(hash_password.as_ref().starts_with("$argon2id$v=19$"));

        let result = hash_password.verify_raw_password(raw_password.as_ref()).await;
        assert!(result.is_ok());

    }

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(pub String);

    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary(g: &mut Gen) -> Self {
            let seed: u64 = g.size() as u64;
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            let password = FakePassword(8..30).fake_with_rng(&mut rng);
            Self(password)
        }
    }

    // updated!
    #[tokio::test]
    #[quickcheck_macros::quickcheck]
    async fn valid_passwords_are_parsed_successfully(valid_password:
                                                     ValidPasswordFixture) -> bool {
        HashedPassword::parse(valid_password.0).await.is_ok()
    }
}