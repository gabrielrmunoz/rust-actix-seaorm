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
    pub role: String,
}
