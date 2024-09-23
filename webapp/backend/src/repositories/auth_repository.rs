use crate::domains::auth_service::AuthRepository;
use crate::errors::AppError;
use crate::models::user::{Dispatcher, Session, User};
use sqlx::mysql::MySqlPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
#[derive(Debug, Clone)]
pub struct AuthRepositoryImpl {
    pool: MySqlPool,
    session_cache: Arc<RwLock<HashMap<String, Session>>>, // メモリ上のセッションキャッシュ
}
impl AuthRepositoryImpl {
    pub fn new(pool: MySqlPool) -> Self {
        AuthRepositoryImpl {
            pool,
            session_cache: Arc::new(RwLock::new(HashMap::new())), // 初期化
        }
    }
}
impl AuthRepository for AuthRepositoryImpl {
    async fn find_user_by_id(&self, id: i32) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
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
    // メモリ上にセッションを作成する
    async fn create_session(&self, user_id: i32, session_token: &str) -> Result<(), AppError> {
        let session = Session {
            id: 0, // メモリ上では不要。データベース使用時にだけ必要。
            user_id,
            session_token: session_token.to_string(),
            is_valid: true, // デフォルトで有効に設定
        };
        // メモリ上のキャッシュに保存
        let mut cache = self.session_cache.write().await;
        cache.insert(session_token.to_string(), session);
        // データベースにも保存する場合
        sqlx::query("INSERT INTO sessions (user_id, session_token, is_valid) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(session_token)
            .bind(true) // デフォルトでは有効なセッションを挿入
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    // メモリ上のセッションを削除する
    async fn delete_session(&self, session_token: &str) -> Result<(), AppError> {
        // メモリ上から削除
        let mut cache = self.session_cache.write().await;
        cache.remove(session_token);
        // データベースで無効化
        sqlx::query("UPDATE sessions SET is_valid = false WHERE session_token = ?")
            .bind(session_token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    // メモリ上のセッションを取得する
    async fn find_session_by_session_token(
        &self,
        session_token: &str,
    ) -> Result<Session, AppError> {
        let cache = self.session_cache.read().await;
        // まずメモリ上で検索
        if let Some(session) = cache.get(session_token) {
            if session.is_valid {
                return Ok(session.clone());
            } else {
                return Err(AppError::NotFound); // 無効なセッション
            }
        }
        // メモリに無ければデータベースで検索
        if let Some(session) = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE session_token = ? AND is_valid = true",
        )
        .bind(session_token)
        .fetch_optional(&self.pool)
        .await?
        {
            let mut cache = self.session_cache.write().await;
            cache.insert(session_token.to_string(), session.clone()); // メモリにキャッシュ
            Ok(session)
        } else {
            Err(AppError::NotFound) // セッションが見つからない、もしくは無効
        }
    }
    async fn find_dispatcher_by_id(&self, id: i32) -> Result<Option<Dispatcher>, AppError> {
        let dispatcher = sqlx::query_as::<_, Dispatcher>("SELECT * FROM dispatchers WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(dispatcher)
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
}
