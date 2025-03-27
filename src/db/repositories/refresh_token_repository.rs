use crate::db::models::{
    RefreshTokenActiveModel, RefreshTokenColumn, RefreshTokenEntity, RefreshTokenModel,
};
use chrono::Local;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use std::sync::Arc;

pub struct RefreshTokenRepository {
    db: Arc<DatabaseConnection>,
}

impl RefreshTokenRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<RefreshTokenModel>, DbErr> {
        RefreshTokenEntity::find_by_id(id)
            .one(self.db.as_ref())
            .await
    }

    pub async fn find_by_refresh_token(
        &self,
        refresh_token: String,
    ) -> Result<Option<RefreshTokenModel>, DbErr> {
        RefreshTokenEntity::find()
            .filter(RefreshTokenColumn::RefreshToken.eq(refresh_token))
            .filter(RefreshTokenColumn::RevokedOn.is_null())
            .one(self.db.as_ref())
            .await
    }

    pub async fn find_by_user_id(&self, user_id: i32) -> Result<Option<RefreshTokenModel>, DbErr> {
        RefreshTokenEntity::find()
            .filter(RefreshTokenColumn::UserId.eq(user_id))
            .filter(RefreshTokenColumn::RevokedOn.is_null())
            .one(self.db.as_ref())
            .await
    }

    pub async fn create(&self, model: RefreshTokenActiveModel) -> Result<RefreshTokenModel, DbErr> {
        model.insert(self.db.as_ref()).await
    }

    pub async fn revoke(&self, id: i32) -> Result<Option<RefreshTokenModel>, DbErr> {
        let token = self.find_by_id(id).await?;
        let now = Local::now().naive_local();

        if let Some(token) = token {
            let mut active_model: RefreshTokenActiveModel = token.into();
            active_model.revoked_on = Set(Some(now));

            Ok(Some(active_model.update(self.db.as_ref()).await?))
        } else {
            Ok(None)
        }
    }
}
