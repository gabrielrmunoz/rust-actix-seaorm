use once_cell::sync::Lazy;
use regex::Regex;
use validator::ValidationError;

use crate::auth::UserRole;

pub static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\+\d{1,3})?-\d{6,14}$").unwrap());

pub fn validate_no_spaces(username: &str) -> Result<(), ValidationError> {
    if username.contains(' ') {
        let mut error = ValidationError::new("no_spaces");
        error.message = Some("Username cannot contain spaces".into());
        return Err(error);
    }
    Ok(())
}

pub fn validate_role(role: &str) -> Result<(), ValidationError> {
    if UserRole::is_valid_role(role) {
        Ok(())
    } else {
        let mut error = ValidationError::new("invalid_role");
        error.message = Some(format!("Role must be one of: admin, user, guest").into());
        Err(error)
    }
}
