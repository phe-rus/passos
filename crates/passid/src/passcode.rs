use argon2::Argon2;
use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};
use thiserror::Error;

pub const MIN_LEN: usize = 6;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PasscodeError {
    #[error("passcode must be at least 6 digits")]
    TooShort,
    #[error("passcode may only contain digits")]
    NotDigits,
    #[error("passcode is too simple")]
    TooSimple,
    #[error("hashing failed")]
    Hash,
}

/// Reject weak passcodes: too short, non-digits, repeated or sequential digits.
pub fn validate(raw: &str) -> Result<(), PasscodeError> {
    if !raw.chars().all(|c| c.is_ascii_digit()) {
        return Err(PasscodeError::NotDigits);
    }
    if raw.len() < MIN_LEN {
        return Err(PasscodeError::TooShort);
    }
    let d: Vec<i8> = raw.bytes().map(|b| (b - b'0') as i8).collect();
    let all_same = d.windows(2).all(|w| w[0] == w[1]);
    let ascending = d.windows(2).all(|w| w[1] - w[0] == 1);
    let descending = d.windows(2).all(|w| w[0] - w[1] == 1);
    if all_same || ascending || descending {
        return Err(PasscodeError::TooSimple);
    }
    Ok(())
}

/// Validate and hash a passcode with Argon2id. Returns a PHC string to store.
pub fn hash(raw: &str) -> Result<String, PasscodeError> {
    validate(raw)?;
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(raw.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| PasscodeError::Hash)
}

/// Check a passcode against a stored PHC string. Any error counts as a mismatch.
pub fn verify(raw: &str, phc: &str) -> bool {
    match PasswordHash::new(phc) {
        Ok(parsed) => Argon2::default()
            .verify_password(raw.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_weak() {
        assert_eq!(validate("123"), Err(PasscodeError::TooShort));
        assert_eq!(validate("12ab56"), Err(PasscodeError::NotDigits));
        assert_eq!(validate("111111"), Err(PasscodeError::TooSimple));
        assert_eq!(validate("123456"), Err(PasscodeError::TooSimple));
        assert_eq!(validate("654321"), Err(PasscodeError::TooSimple));
    }

    #[test]
    fn accepts_ok() {
        assert!(validate("482915").is_ok());
    }

    #[test]
    fn hash_and_verify() {
        let h = hash("482915").unwrap();
        assert!(h.starts_with("$argon2id$"));
        assert!(verify("482915", &h));
        assert!(!verify("482916", &h));
        assert!(!verify("482915", "garbage"));
    }

    #[test]
    fn salts_differ() {
        assert_ne!(hash("482915").unwrap(), hash("482915").unwrap());
    }
}
