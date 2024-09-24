use super::{
    auth_service::AuthRepository, dto::order::OrderDto, map_service::MapRepository,
    tow_truck_service::TowTruckRepository,
};
use crate::models::tow_truck::TowTruck;
use crate::models::user::Dispatcher;
use crate::models::user::User;
use crate::{errors::AppError, models::order::Order};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
pub trait OrderRepository {
    async fn find_order_by_id(&self, id: i32) -> Result<Order, AppError>;
    async fn update_order_status(&self, order_id: i32, status: &str) -> Result<(), AppError>;
    async fn get_paginated_orders(
        &self,
        page: i32,
        page_size: i32,
        sort_by: Option<String>,
        sort_order: Option<String>,
        status: Option<String>,
        area: Option<i32>,
    ) -> Result<Vec<Order>, AppError>;
    async fn create_order(
        &self,
        customer_id: i32,
        node_id: i32,
        car_value: f64,
    ) -> Result<(), AppError>;
    async fn update_order_dispatched(
        &self,
        id: i32,
        dispatcher_id: i32,
        tow_truck_id: i32,
    ) -> Result<(), AppError>;
    async fn create_completed_order(
        &self,
        order_id: i32,
        tow_truck_id: i32,
        completed_time: DateTime<Utc>,
    ) -> Result<(), AppError>;
}

#[derive(Debug)]
pub struct OrderService<
    T: OrderRepository + std::fmt::Debug,
    U: TowTruckRepository + std::fmt::Debug,
    V: AuthRepository + std::fmt::Debug,
    W: MapRepository + std::fmt::Debug,
> {
    order_repository: T,
    tow_truck_repository: U,
    auth_repository: V,
    map_repository: W,
}

impl<
        T: OrderRepository + std::fmt::Debug,
        U: TowTruckRepository + std::fmt::Debug,
        V: AuthRepository + std::fmt::Debug,
        W: MapRepository + std::fmt::Debug,
    > OrderService<T, U, V, W>
{
    pub fn new(
        order_repository: T,
        tow_truck_repository: U,
        auth_repository: V,
        map_repository: W,
    ) -> Self {
        OrderService {
            order_repository,
            tow_truck_repository,
            auth_repository,
            map_repository,
        }
    }

    pub async fn update_order_status(&self, order_id: i32, status: &str) -> Result<(), AppError> {
        self.order_repository
            .update_order_status(order_id, status)
            .await
    }

    pub async fn get_order_by_id(&self, id: i32) -> Result<OrderDto, AppError> {
        let order = self.order_repository.find_order_by_id(id).await?;

        let client_username = self
            .auth_repository
            .find_user_by_id(order.client_id)
            .await
            .unwrap()
            .unwrap()
            .username;

        let dispatcher = match order.dispatcher_id {
            Some(dispatcher_id) => self
                .auth_repository
                .find_dispatcher_by_id(dispatcher_id)
                .await
                .unwrap(),
            None => None,
        };

        let (dispatcher_user_id, dispatcher_username) = match dispatcher {
            Some(dispatcher) => (
                Some(dispatcher.user_id),
                Some(
                    self.auth_repository
                        .find_user_by_id(dispatcher.user_id)
                        .await
                        .unwrap()
                        .unwrap()
                        .username,
                ),
            ),
            None => (None, None),
        };

        let tow_truck = match order.tow_truck_id {
            Some(tow_truck_id) => self
                .tow_truck_repository
                .find_tow_truck_by_id(tow_truck_id)
                .await
                .unwrap(),
            None => None,
        };

        let (driver_user_id, driver_username) = match tow_truck {
            Some(tow_truck) => (
                Some(tow_truck.driver_id),
                Some(
                    self.auth_repository
                        .find_user_by_id(tow_truck.driver_id)
                        .await
                        .unwrap()
                        .unwrap()
                        .username,
                ),
            ),
            None => (None, None),
        };

        let area_id = order.area_id;

        Ok(OrderDto {
            id: order.id,
            client_id: order.client_id,
            client_username: Some(client_username),
            dispatcher_user_id,
            dispatcher_username,
            driver_user_id,
            driver_username,
            area_id,
            dispatcher_id: order.dispatcher_id,
            tow_truck_id: order.tow_truck_id,
            status: order.status,
            node_id: order.node_id,
            car_value: order.car_value,
            order_time: order.order_time,
            completed_time: order.completed_time,
        })
    }

    pub async fn get_paginated_orders(
        &self,
        page: i32,
        page_size: i32,
        sort_by: Option<String>,
        sort_order: Option<String>,
        status: Option<String>,
        area: Option<i32>,
    ) -> Result<Vec<OrderDto>, AppError> {
        let orders = self
            .order_repository
            .get_paginated_orders(page, page_size, sort_by, sort_order, status, area)
            .await?;
        // すべてのIDを収集
        let client_ids: Vec<i32> = orders.iter().map(|order| order.client_id).collect();
        let dispatcher_ids: Vec<i32> = orders
            .iter()
            .filter_map(|order| order.dispatcher_id)
            .collect();
        let tow_truck_ids: Vec<i32> = orders
            .iter()
            .filter_map(|order| order.tow_truck_id)
            .collect();
        // バルクでクライアント、ディスパッチャー、トウトラックを取得
        let clients = self.auth_repository.find_users_by_ids(&client_ids).await?;
        let dispatchers = self
            .auth_repository
            .find_dispatchers_by_ids(&dispatcher_ids)
            .await?;
        let tow_trucks = self
            .tow_truck_repository
            .find_tow_truck_by_ids(&tow_truck_ids)
            .await?;
        // IDをキーにしたHashMapを作成
        let client_map: HashMap<i32, User> =
            clients.into_iter().map(|user| (user.id, user)).collect();
        let dispatcher_map: HashMap<i32, Dispatcher> = dispatchers
            .into_iter()
            .map(|dispatcher| (dispatcher.user_id, dispatcher))
            .collect();
        let tow_truck_map: HashMap<i32, TowTruck> = tow_trucks
            .into_iter()
            .map(|tow_truck| (tow_truck.id, tow_truck))
            .collect();
        let mut results = Vec::new();
        for order in orders {
            // クライアント情報を取得
            let client_username = client_map
                .get(&order.client_id)
                .map(|client| client.username.clone());
            // ディスパッチャー情報を取得
            let (dispatcher_user_id, dispatcher_username) = match order.dispatcher_id {
                Some(dispatcher_id) => {
                    if let Some(dispatcher) = dispatcher_map.get(&dispatcher_id) {
                        let dispatcher_user = client_map.get(&dispatcher.user_id);
                        (
                            Some(dispatcher.user_id),
                            dispatcher_user.map(|user| user.username.clone()),
                        )
                    } else {
                        (None, None)
                    }
                }
                None => (None, None),
            };
            // トウトラック情報を取得
            let (driver_user_id, driver_username) = match order.tow_truck_id {
                Some(tow_truck_id) => {
                    if let Some(tow_truck) = tow_truck_map.get(&tow_truck_id) {
                        let driver_user = client_map.get(&tow_truck.driver_id);
                        (
                            Some(tow_truck.driver_id),
                            driver_user.map(|user| user.username.clone()),
                        )
                    } else {
                        (None, None)
                    }
                }
                None => (None, None),
            };
            results.push(OrderDto {
                id: order.id,
                client_id: order.client_id,
                client_username,
                dispatcher_id: order.dispatcher_id,
                dispatcher_user_id,
                dispatcher_username,
                tow_truck_id: order.tow_truck_id,
                driver_user_id,
                driver_username,
                area_id: order.area_id,
                status: order.status,
                node_id: order.node_id,
                car_value: order.car_value,
                order_time: order.order_time,
                completed_time: order.completed_time,
            });
        }
        Ok(results)
    }

    pub async fn create_client_order(
        &self,
        client_id: i32,
        node_id: i32,
        car_value: f64,
    ) -> Result<(), AppError> {
        match self
            .order_repository
            .create_order(client_id, node_id, car_value)
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(AppError::BadRequest),
        }
    }

    pub async fn create_dispatcher_order(
        &self,
        order_id: i32,
        dispatcher_id: i32,
        tow_truck_id: i32,
        order_time: DateTime<Utc>,
    ) -> Result<(), AppError> {
        if (self
            .order_repository
            .create_completed_order(order_id, tow_truck_id, order_time)
            .await)
            .is_err()
        {
            return Err(AppError::BadRequest);
        }

        self.order_repository
            .update_order_dispatched(order_id, dispatcher_id, tow_truck_id)
            .await?;

        self.tow_truck_repository
            .update_status(tow_truck_id, "busy")
            .await?;

        Ok(())
    }
}
