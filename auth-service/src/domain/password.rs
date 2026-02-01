

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Password (pub String);

impl Password {
    pub fn parse(password: String) -> Result<Password, String> {
        if validate_password(&password) {
            Ok(Self(password))
        } else {
            Err("Failed to parse string to a Password type".to_owned())
        }
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

fn validate_password(s: &str) -> bool {
    s.len() >= 8
}

#[cfg(test)]
mod tests {
    use std::ops::Range;
    use super::*;
    use fake::faker::internet::en::Password;
    use fake::Fake;
    use quickcheck_macros::quickcheck;

    #[test]
    fn empty_password_is_invalid() {
        assert!(!validate_password(""));
    }

    #[test]
    fn short_password_is_invalid() {
        assert!(!validate_password("123"));
    }

    #[test]
    fn long_password_is_valid() {
        assert!(validate_password("123456789"));
    }

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(pub String);

    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let password = Password(8..25).fake();
            Self(password)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_password_are_all_valid(pass: String){
        let password: String = Password(8..25).fake();
        assert!(validate_password(&password));
    }

}