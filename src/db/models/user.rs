use sea_orm::{entity::prelude::*, sqlx::types::chrono::NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tbl_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,

    #[serde(skip_serializing)]
    pub password: String,

    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub role: String,
    pub created_on: NaiveDateTime,
    pub updated_on: NaiveDateTime,
    pub deleted_on: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
