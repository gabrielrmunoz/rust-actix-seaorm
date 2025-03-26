use once_cell::sync::Lazy;
use regex::Regex;
use validator::ValidationError;

use crate::auth::UserRole;

pub static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\+\d{1,3})?-\d{6,14}$").unwrap());

pub static PASSWORD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{10,20}$").unwrap()
});

pub fn validate_no_spaces(username: &str) -> Result<(), ValidationError> {
    if username.contains(' ') {
        let mut error = ValidationError::new("no_spaces");
        error.message = Some("Username cannot contain spaces".into());
        return Err(error);
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if PASSWORD_REGEX.is_match(password) {
        Ok(())
    } else {
        let mut error = ValidationError::new("invalid_password");
        error.message = Some("Password must be 10-20 characters long and include at least one uppercase letter, one lowercase letter, one number, and one special character (@$!%*?&)".into());
        Err(error)
    }
}

pub fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    if PHONE_REGEX.is_match(phone) {
        Ok(())
    } else {
        let mut error = ValidationError::new("invalid_phone");
        error.message = Some("Phone number must be in the format +123-1234567890".into());
        Err(error)
    }
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
