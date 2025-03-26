use crate::db::models::{Model, UserActiveModel, UserColumn, UserEntity};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use sea_orm::{DeleteResult, prelude::*};
use std::sync::Arc;

pub struct UserRepository {
    db: Arc<DatabaseConnection>,
}

impl UserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_all(&self, include_deleted: bool) -> Result<Vec<Model>, DbErr> {
        let mut query = UserEntity::find();

        if !include_deleted {
            query = query.filter(UserColumn::DeletedOn.is_null());
        }

        query.all(self.db.as_ref()).await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<Model>, DbErr> {
        UserEntity::find_by_id(id).one(self.db.as_ref()).await
    }

    pub async fn find_by_username(&self, username: String) -> Result<Option<Model>, DbErr> {
        UserEntity::find()
            .filter(UserColumn::Username.eq(username))
            .one(self.db.as_ref())
            .await
    }

    pub async fn find_by_email(&self, email: String) -> Result<Option<Model>, DbErr> {
        UserEntity::find()
            .filter(UserColumn::Email.eq(email))
            .one(self.db.as_ref())
            .await
    }

    pub async fn find_by_phone(&self, phone: String) -> Result<Option<Model>, DbErr> {
        UserEntity::find()
            .filter(UserColumn::Phone.eq(phone))
            .one(self.db.as_ref())
            .await
    }

    pub async fn create(&self, model: UserActiveModel) -> Result<Model, DbErr> {
        model.insert(self.db.as_ref()).await
    }

    pub async fn update(&self, model: UserActiveModel) -> Result<Model, DbErr> {
        model.update(self.db.as_ref()).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        UserEntity::delete_by_id(id).exec(self.db.as_ref()).await
    }

    pub async fn soft_delete(&self, id: i32, now: DateTime) -> Result<Option<Model>, DbErr> {
        let user = self.find_by_id(id).await?;

        if let Some(user) = user {
            let mut active_model: UserActiveModel = user.into();
            active_model.deleted_on = Set(Some(now));
            active_model.updated_on = Set(now);

            Ok(Some(active_model.update(self.db.as_ref()).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn restore(&self, id: i32, now: DateTime) -> Result<Option<Model>, DbErr> {
        let user = self.find_by_id(id).await?;

        if let Some(user) = user {
            let mut active_model: UserActiveModel = user.into();
            active_model.deleted_on = Set(None);
            active_model.updated_on = Set(now);

            Ok(Some(active_model.update(self.db.as_ref()).await?))
        } else {
            Ok(None)
        }
    }
}
