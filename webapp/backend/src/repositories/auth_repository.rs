use crate::errors::AppError;
use crate::models::user::{Dispatcher, User};
use crate::{domains::auth_service::AuthRepository, models::user::Session};
use sqlx::mysql::MySqlPool;
use std::collections::HashMap;
#[derive(Debug)]
pub struct AuthRepositoryImpl {
    pool: MySqlPool,
}
impl AuthRepositoryImpl {
    pub fn new(pool: MySqlPool) -> Self {
        AuthRepositoryImpl { pool }
    }
}
impl AuthRepository for AuthRepositoryImpl {
    // 既存の find_user_by_id メソッド
    async fn find_user_by_id(&self, id: i32) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }
    // 追加: 複数のユーザーを一度に取得するメソッド
    async fn find_users_by_ids(&self, ids: &[i32]) -> Result<Vec<User>, AppError> {
        if ids.is_empty() {
            return Ok(vec![]); // 空のIDリストに対しては空の結果を返す
        }
        // プレースホルダーの生成
        let query_placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        // クエリ文字列を作成
        let query = format!("SELECT * FROM users WHERE id IN ({})", query_placeholders);
        // クエリを実行し、IDリストをバインド
        let mut query_builder = sqlx::query_as::<_, User>(&query);
        for id in ids {
            query_builder = query_builder.bind(id);
        }
        // クエリの実行
        let users = query_builder.fetch_all(&self.pool).await?;
        Ok(users)
    }
    async fn find_user_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }
    async fn find_profile_image_name_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Option<String>, AppError> {
        let profile_image_name = sqlx::query_scalar("SELECT profile_image FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(profile_image_name)
    }
    async fn create_user(
        &self,
        username: &str,
        password: &str,
        role: &str,
    ) -> Result<(), AppError> {
        sqlx::query("INSERT INTO users (username, password, role) VALUES (?, ?, ?)")
            .bind(username)
            .bind(password)
            .bind(role)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    async fn create_session(&self, user_id: i32, session_token: &str) -> Result<(), AppError> {
        sqlx::query("INSERT INTO sessions (user_id, session_token) VALUES (?, ?)")
            .bind(user_id)
            .bind(session_token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    async fn delete_session(&self, session_token: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM sessions WHERE session_token = ?")
            .bind(session_token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    async fn find_session_by_session_token(
        &self,
        session_token: &str,
    ) -> Result<Session, AppError> {
        let session =
            sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE session_token = ?")
                .bind(session_token)
                .fetch_one(&self.pool)
                .await?;
        Ok(session)
    }
    async fn find_dispatcher_by_id(&self, id: i32) -> Result<Option<Dispatcher>, AppError> {
        let dispatcher = sqlx::query_as::<_, Dispatcher>("SELECT * FROM dispatchers WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(dispatcher)
    }
    // 追加: 複数のディスパッチャーを一度に取得するメソッド
    async fn find_dispatchers_by_ids(&self, ids: &[i32]) -> Result<Vec<Dispatcher>, AppError> {
        if ids.is_empty() {
            return Ok(vec![]); // 空のIDリストに対しては空の結果を返す
        }
        // プレースホルダーの生成
        let query_placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        // クエリ文字列を作成
        let query = format!(
            "SELECT * FROM dispatchers WHERE id IN ({})",
            query_placeholders
        );
        // クエリを実行し、IDリストをバインド
        let mut query_builder = sqlx::query_as::<_, Dispatcher>(&query);
        for id in ids {
            query_builder = query_builder.bind(id);
        }
        // クエリの実行
        let dispatchers = query_builder.fetch_all(&self.pool).await?;
        Ok(dispatchers)
    }

    async fn find_dispatcher_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Option<Dispatcher>, AppError> {
        let dispatcher =
            sqlx::query_as::<_, Dispatcher>("SELECT * FROM dispatchers WHERE user_id = ?")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(dispatcher)
    }
    async fn create_dispatcher(&self, user_id: i32, area_id: i32) -> Result<(), AppError> {
        sqlx::query("INSERT INTO dispatchers (user_id, area_id) VALUES (?, ?)")
            .bind(user_id)
            .bind(area_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_user_by_ids(&self, ids: &[i32]) -> Result<Vec<User>, AppError> {
        todo!()
    }
}
