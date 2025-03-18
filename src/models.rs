use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tbl_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub created_on: String,
    pub updated_on: String,
    pub deleted_on: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
impl Entity {
    #[allow(dead_code)]
    pub fn insert(model: Model) -> ActiveModel {
        ActiveModel {
            username: Set(model.username),
            first_name: Set(model.first_name),
            last_name: Set(model.last_name),
            email: Set(model.email),
            phone: Set(model.phone),
            created_on: Set(model.created_on),
            updated_on: Set(model.updated_on),
            deleted_on: Set(model.deleted_on),
            ..Default::default()
        }
    }
}
