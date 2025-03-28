use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct User {
    pub id: Option<i32>,

    #[validate(length(min = 3, message = "Username must be at least 3 characters"))]
    pub username: String,

    pub first_name: Option<String>,
    pub last_name: Option<String>,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    pub phone: Option<String>,
}

impl User {
    pub fn new(username: String, email: String) -> Self {
        Self {
            id: None,
            username,
            first_name: None,
            last_name: None,
            email,
            phone: None,
        }
    }

    pub fn full_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => format!("{} {}", first, last),
            (Some(first), None) => first.clone(),
            (None, Some(last)) => last.clone(),
            (None, None) => self.username.clone(),
        }
    }

    pub fn validate_business_rules(&self) -> Result<(), String> {
        if self.username.contains(' ') {
            return Err("Username cannot contain spaces".to_string());
        }

        Ok(())
    }
}
