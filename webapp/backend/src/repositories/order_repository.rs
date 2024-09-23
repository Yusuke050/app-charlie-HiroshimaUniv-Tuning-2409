use crate::domains::order_service::OrderRepository;
use crate::errors::AppError;
use crate::models::order::Order;
use chrono::{DateTime, Utc};
use sqlx::mysql::MySqlPool;

#[derive(Debug)]
pub struct OrderRepositoryImpl {
    pool: MySqlPool,
}

impl OrderRepositoryImpl {
    pub fn new(pool: MySqlPool) -> Self {
        OrderRepositoryImpl { pool }
    }
}

impl OrderRepository for OrderRepositoryImpl {
    async fn find_order_by_id(&self, id: i32) -> Result<Order, AppError> {
        let order = sqlx::query_as::<_, Order>(
            "SELECT 
                *
            FROM
                orders 
            WHERE
                id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(order)
    }

    async fn update_order_status(&self, order_id: i32, status: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE orders SET status = ? WHERE id = ?")
            .bind(status)
            .bind(order_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_paginated_orders(
        &self,
        page: i32,
        page_size: i32,
        sort_by: Option<String>,
        sort_order: Option<String>,
        status: Option<String>,
        area: Option<i32>,
    ) -> Result<Vec<Order>, AppError> {
        let offset = page * page_size;
        let order_clause = format!(
            "ORDER BY {} {}",
            match sort_by.as_deref() {
                Some("car_value") => "o.car_value",
                Some("status") => "o.status",
                Some("order_time") => "o.order_time",
                _ => "o.order_time",
            },
            match sort_order.as_deref() {
                Some("DESC") => "DESC",
                Some("desc") => "DESC",
                _ => "ASC",
            }
        );

        let where_clause = match (status.clone(), area) {
            (Some(_), Some(_)) => "WHERE o.status = ? AND o.area_id = ?".to_string(),
            (None, Some(_)) => "WHERE o.area_id = ?".to_string(),
            (Some(_), None) => "WHERE o.status = ?".to_string(),
            _ => "".to_string(),
        };

        let sql = format!(
            "SELECT 
                o.id, 
                o.client_id, 
                o.dispatcher_id, 
                o.tow_truck_id, 
                o.status, 
                o.node_id, 
                o.car_value, 
                o.order_time, 
                o.completed_time
            FROM
                orders o
            {} 
            {} 
            LIMIT ? 
            OFFSET ?",
            where_clause, order_clause
        );

        let orders = match (status, area) {
            (Some(status), Some(area)) => {
                sqlx::query_as::<_, Order>(&sql)
                    .bind(status)
                    .bind(area)
                    .bind(page_size)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await?
            }
            (None, Some(area)) => {
                sqlx::query_as::<_, Order>(&sql)
                    .bind(area)
                    .bind(page_size)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await?
            }
            (Some(status), None) => {
                sqlx::query_as::<_, Order>(&sql)
                    .bind(status)
                    .bind(page_size)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await?
            }
            _ => {
                sqlx::query_as::<_, Order>(&sql)
                    .bind(page_size)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await?
            }
        };

        Ok(orders)
    }

    async fn create_order(
        &self,
        client_id: i32,
        node_id: i32,
        car_value: f64,
    ) -> Result<(), AppError> {
        // node_id に対応する area_id を取得
        let area_id: i32 = sqlx::query_scalar("SELECT area_id FROM nodes WHERE id = ?")
            .bind(node_id)
            .fetch_one(&self.pool)
            .await?;
        
        // orders テーブルに新しいレコードを挿入
        sqlx::query("INSERT INTO orders (client_id, node_id, area_id, status, car_value) VALUES (?, ?, ?, 'pending', ?)")
            .bind(client_id)
            .bind(node_id)
            .bind(area_id)
            .bind(car_value)
            .execute(&self.pool)
            .await?;
    
        Ok(())
    }

    async fn update_order_dispatched(
        &self,
        id: i32,
        dispatcher_id: i32,
        tow_truck_id: i32,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE orders SET dispatcher_id = ?, tow_truck_id = ?, status = 'dispatched' WHERE id = ?",
        )
        .bind(dispatcher_id)
        .bind(tow_truck_id)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
// /order/dispatcher
    async fn create_completed_order(
        &self,
        order_id: i32,
        tow_truck_id: i32,
        completed_time: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query("INSERT INTO completed_orders (order_id, tow_truck_id, completed_time) VALUES (?, ?, ?)")
            .bind(order_id)
            .bind(tow_truck_id)
            .bind(completed_time)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
