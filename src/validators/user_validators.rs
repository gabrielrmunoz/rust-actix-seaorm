use once_cell::sync::Lazy;
use regex::Regex;
use validator::ValidationError;

pub static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\+\d{1,3})?-\d{6,14}$").unwrap());

pub fn validate_no_spaces(username: &str) -> Result<(), ValidationError> {
    if username.contains(' ') {
        let mut error = ValidationError::new("no_spaces");
        error.message = Some("Username cannot contain spaces".into());
        return Err(error);
    }
    Ok(())
}
