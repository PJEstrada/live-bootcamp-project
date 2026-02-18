use serde::Deserialize;
use validator::{Validate, ValidateEmail, ValidationError};
#[derive(Debug, Clone, PartialEq, Hash, Eq, Deserialize)]
pub struct Email (pub String);

impl Email {
    pub fn parse (email_str: String) -> Result<Email, String> {
        if email_str.validate_email() {
            Ok(Self(email_str))
        } else {
            Err(format!("Invalid email: {}", email_str))
        }
    }

}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use fake::Fake;
    use quickcheck::Gen;
    use fake::faker::internet::en::SafeEmail;
    use rand::SeedableRng;
    use super::*;
    #[test]
    fn test_parse_email() {
        // Invalid email case
        assert!(Email::parse("asdasd".to_string()).is_err());
        // Valid email case
        assert!(Email::parse("test@test.com".to_string()).is_ok());
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut Gen) -> Self {
            let seed = g.size() as u64;
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }
    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        Email::parse(valid_email.0).is_ok()
    }
}