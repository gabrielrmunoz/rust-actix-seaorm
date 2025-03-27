use crate::db::models::{UserActiveModel, UserColumn, UserEntity, UserModel};
use chrono::Local;
use sea_orm::DeleteResult;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

pub struct UserRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> UserRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_all(&self, include_deleted: bool) -> Result<Vec<UserModel>, DbErr> {
        let mut query = UserEntity::find();

        if !include_deleted {
            query = query.filter(UserColumn::DeletedOn.is_null());
        }

        query.all(self.db).await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<UserModel>, DbErr> {
        UserEntity::find_by_id(id).one(self.db).await
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<UserModel>, DbErr> {
        UserEntity::find()
            .filter(UserColumn::Username.eq(username))
            .one(self.db)
            .await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserModel>, DbErr> {
        UserEntity::find()
            .filter(UserColumn::Email.eq(email))
            .one(self.db)
            .await
    }

    pub async fn find_by_phone(&self, phone: &str) -> Result<Option<UserModel>, DbErr> {
        UserEntity::find()
            .filter(UserColumn::Phone.eq(phone))
            .one(self.db)
            .await
    }

    pub async fn create(&self, model: UserActiveModel) -> Result<UserModel, DbErr> {
        model.insert(self.db).await
    }

    pub async fn update(&self, model: UserActiveModel) -> Result<UserModel, DbErr> {
        model.update(self.db).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        UserEntity::delete_by_id(id).exec(self.db).await
    }

    pub async fn soft_delete(&self, id: i32) -> Result<Option<UserModel>, DbErr> {
        let user = self.find_by_id(id).await?;
        let now = Local::now().naive_local();

        if let Some(user) = user {
            let mut user_active_model: UserActiveModel = user.into();
            user_active_model.deleted_on = Set(Some(now));
            user_active_model.updated_on = Set(now);

            Ok(Some(user_active_model.update(self.db).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn restore(&self, id: i32) -> Result<Option<UserModel>, DbErr> {
        let user = self.find_by_id(id).await?;
        let now = Local::now().naive_local();

        if let Some(user) = user {
            let mut user_active_model: UserActiveModel = user.into();
            user_active_model.deleted_on = Set(None);
            user_active_model.updated_on = Set(now);

            Ok(Some(user_active_model.update(self.db).await?))
        } else {
            Ok(None)
        }
    }
}
