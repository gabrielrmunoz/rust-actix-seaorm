use crate::db::models::{
    RefreshTokenActiveModel, RefreshTokenColumn, RefreshTokenEntity, RefreshTokenModel,
};
use chrono::Local;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

pub struct RefreshTokenRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> RefreshTokenRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<RefreshTokenModel>, DbErr> {
        RefreshTokenEntity::find_by_id(id).one(self.db).await
    }

    pub async fn find_by_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<Option<RefreshTokenModel>, DbErr> {
        RefreshTokenEntity::find()
            .filter(RefreshTokenColumn::RefreshToken.eq(refresh_token))
            .filter(RefreshTokenColumn::RevokedOn.is_null())
            .one(self.db)
            .await
    }

    pub async fn find_by_user_id(&self, user_id: i32) -> Result<Option<RefreshTokenModel>, DbErr> {
        RefreshTokenEntity::find()
            .filter(RefreshTokenColumn::UserId.eq(user_id))
            .filter(RefreshTokenColumn::RevokedOn.is_null())
            .one(self.db)
            .await
    }

    pub async fn create(&self, model: RefreshTokenActiveModel) -> Result<RefreshTokenModel, DbErr> {
        model.insert(self.db).await
    }

    pub async fn revoke(&self, id: i32) -> Result<Option<RefreshTokenModel>, DbErr> {
        let token = self.find_by_id(id).await?;
        let now = Local::now().naive_local();

        if let Some(token) = token {
            let mut refresh_token_active_model: RefreshTokenActiveModel = token.into();
            refresh_token_active_model.revoked_on = Set(Some(now));

            Ok(Some(refresh_token_active_model.update(self.db).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn revoke_by_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<Option<RefreshTokenModel>, DbErr> {
        let token = self.find_by_refresh_token(refresh_token).await?;
        let now = Local::now().naive_local();

        if let Some(token) = token {
            let mut refresh_token_active_model: RefreshTokenActiveModel = token.into();
            refresh_token_active_model.revoked_on = Set(Some(now));

            Ok(Some(refresh_token_active_model.update(self.db).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn revoke_all_for_user(&self, user_id: i32) -> Result<Vec<RefreshTokenModel>, DbErr> {
        let tokens = RefreshTokenEntity::find()
            .filter(RefreshTokenColumn::UserId.eq(user_id))
            .filter(RefreshTokenColumn::RevokedOn.is_null())
            .all(self.db)
            .await?;

        let now = Local::now().naive_local();
        let mut revoked_tokens = Vec::with_capacity(tokens.len());

        for token in tokens {
            let mut refresh_token_active_model: RefreshTokenActiveModel = token.into();
            refresh_token_active_model.revoked_on = Set(Some(now));

            let updated = refresh_token_active_model.update(self.db).await?;
            revoked_tokens.push(updated);
        }

        Ok(revoked_tokens)
    }
}
