use sqlx::mysql::MySqlPool;
use std::env;

// pub async fn create_pool() -> MySqlPool {
//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     MySqlPool::connect(&database_url)
//         .await
//         .expect("Failed to create pool")
// }

pub async fn create_pool() -> MySqlPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // 環境変数から接続プールのサイズを取得し、デフォルトを10に設定
    let pool_size: u32 = env::var("DATABASE_POOL_SIZE")
        .unwrap_or_else(|_| "10".to_string())  // デフォルトで10を使用
        .parse()
        .expect("DATABASE_POOL_SIZE must be a valid number");

    MySqlPoolOptions::new()
        .max_connections(pool_size)  // 接続プールの最大サイズを設定
        .connect_timeout(Duration::from_secs(30))  // 接続タイムアウトを設定
        .connect(&database_url)
        .await
        .expect("Failed to create pool")
}
