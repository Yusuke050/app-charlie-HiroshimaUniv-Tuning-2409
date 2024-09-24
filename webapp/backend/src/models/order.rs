use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(FromRow, Clone, Debug)]
pub struct Order {
    pub id: i32,
    pub client_id: i32,
    pub dispatcher_id: Option<i32>,
    pub tow_truck_id: Option<i32>,
    pub status: String,
    pub node_id: i32,
    pub car_value: f64,
    pub order_time: DateTime<Utc>,
    pub completed_time: Option<DateTime<Utc>>,
    pub area_id: i32,
}
