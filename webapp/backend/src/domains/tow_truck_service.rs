use super::dto::tow_truck::TowTruckDto;
use super::map_service::MapRepository;
use super::order_service::OrderRepository;
use crate::errors::AppError;
use crate::models::graph::Graph;
use crate::models::tow_truck::TowTruck;

pub trait TowTruckRepository {
    async fn get_paginated_tow_trucks(
        &self,
        page: i32,
        page_size: i32,
        status: Option<String>,
        area_id: Option<i32>,
    ) -> Result<Vec<TowTruck>, AppError>;
    async fn update_location(&self, truck_id: i32, node_id: i32) -> Result<(), AppError>;
    async fn update_status(&self, truck_id: i32, status: &str) -> Result<(), AppError>;
    async fn find_tow_truck_by_id(&self, id: i32) -> Result<Option<TowTruck>, AppError>;
    async fn find_tow_truck_by_ids(&self, ids: &[i32]) -> Result<Vec<TowTruck>, AppError>;
}

#[derive(Debug)]
pub struct TowTruckService<
    T: TowTruckRepository + std::fmt::Debug,
    U: OrderRepository + std::fmt::Debug,
    V: MapRepository + std::fmt::Debug,
> {
    tow_truck_repository: T,
    order_repository: U,
    map_repository: V,
}

impl<
        T: TowTruckRepository + std::fmt::Debug,
        U: OrderRepository + std::fmt::Debug,
        V: MapRepository + std::fmt::Debug,
    > TowTruckService<T, U, V>
{
    pub fn new(tow_truck_repository: T, order_repository: U, map_repository: V) -> Self {
        TowTruckService {
            tow_truck_repository,
            order_repository,
            map_repository,
        }
    }

    pub async fn get_tow_truck_by_id(&self, id: i32) -> Result<Option<TowTruckDto>, AppError> {
        let tow_truck = self.tow_truck_repository.find_tow_truck_by_id(id).await?;
        Ok(tow_truck.map(TowTruckDto::from_entity))
    }

    pub async fn get_all_tow_trucks(
        &self,
        page: i32,
        page_size: i32,
        status: Option<String>,
        area: Option<i32>,
    ) -> Result<Vec<TowTruckDto>, AppError> {
        let tow_trucks = self
            .tow_truck_repository
            .get_paginated_tow_trucks(page, page_size, status, area)
            .await?;
        let tow_truck_dtos = tow_trucks
            .into_iter()
            .map(TowTruckDto::from_entity)
            .collect();

        Ok(tow_truck_dtos)
    }

    pub async fn update_location(&self, truck_id: i32, node_id: i32) -> Result<(), AppError> {
        self.tow_truck_repository
            .update_location(truck_id, node_id)
            .await?;

        Ok(())
    }

    pub async fn get_nearest_available_tow_trucks(
        &self,
        order_id: i32,
    ) -> Result<Option<TowTruckDto>, AppError> {
        let order = self.order_repository.find_order_by_id(order_id).await?;
        let area_id = self
            .map_repository
            .get_area_id_by_node_id(order.node_id)
            .await?;
        let tow_trucks = self
            .tow_truck_repository
            .get_paginated_tow_trucks(0, -1, Some("available".to_string()), Some(area_id))
            .await?;

        let nodes = self.map_repository.get_all_nodes(Some(area_id)).await?;
        let edges = self.map_repository.get_all_edges(Some(area_id)).await?;

        let mut graph = Graph::new();
        for node in nodes {
            graph.add_node(node);
        }
        for edge in edges {
            graph.add_edge(edge);
        }

        // let sorted_tow_trucks_by_distance = {
        //     let mut tow_trucks_with_distance: Vec<_> = tow_trucks
        //         .into_iter()
        //         .map(|truck| {
        //             let distance = calculate_distance(&graph, truck.node_id, order.node_id);
        //             (distance, truck)
        //         })
        //         .collect();

        //     tow_trucks_with_distance.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        //     tow_trucks_with_distance
        // };

        // let sorted_tow_trucks_by_distance = {
        //     let mut tow_trucks_with_distance: Vec<_> = tow_trucks
        //         .into_iter()
        //         .map(|truck| {
        //             let distance = calculate_distance(&graph, truck.node_id, order.node_id);
        //             (distance, truck)
        //         })
        //         .collect();

        //     tow_trucks_with_distance.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        //     tow_trucks_with_distance
        // };

        let nearest_tow_truck = {
            // ダイクストラ法を使用して、order.node_id（ユーザーがいる位置）から各ノードまでの最短距離を計算
            let distances_from_order = graph.dijkstra(order.node_id);

            // 最短距離とそのトラックを保持するための変数。初期値として非常に大きな距離 (10000001) を設定
            let mut nearest_truck: Option<TowTruck> = None;
            let mut min_distance = 10000001;
            let mut min_truck_id = i32::MAX; // 最小IDを保持するための変数

            for truck in tow_trucks {
                // トラックの位置 (truck.node_id) までの最短距離を取得
                let distance = distances_from_order
                    .get(&truck.node_id)
                    .cloned()
                    .unwrap_or(10000001);

                // 現在の距離が min_distance より小さい場合、または同じ距離でトラックのIDが小さい場合に更新
                if distance < min_distance
                    || (distance == min_distance && truck.node_id < min_truck_id)
                {
                    min_distance = distance;
                    min_truck_id = truck.node_id; // IDも更新
                    nearest_truck = Some(truck);
                }
            }

            // 最短距離が初期値のままかどうかをチェック
            if min_distance == 10000001 {
                None
            } else {
                nearest_truck
            }
        };

        if let Some(truck) = nearest_tow_truck {
            Ok(Some(TowTruckDto::from_entity(truck)))
        } else {
            Ok(None)
        }
    }
}

// fn calculate_distance(graph: &Graph, node_id_1: i32, node_id_2: i32) -> i32 {
//     graph.shortest_path(node_id_1, node_id_2)
// }

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Eq, PartialEq)]
struct State {
    node_id: i32,
    cost: i32,
}

// ダイクストラ法のためのヒープでの比較
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // コストが小さい方を優先するため、reverse
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Graph {
    pub fn dijkstra(&self, start_node_id: i32) -> HashMap<i32, i32> {
        let mut distances: HashMap<i32, i32> = HashMap::new();
        let mut heap = BinaryHeap::new();

        // スタート地点の初期化
        distances.insert(start_node_id, 0);
        heap.push(State {
            node_id: start_node_id,
            cost: 0,
        });

        // ダイクストラ法のメインループ
        while let Some(State { node_id, cost }) = heap.pop() {
            // もし既に最短経路が確定しているならスキップ
            if let Some(&current_cost) = distances.get(&node_id) {
                if cost > current_cost {
                    continue;
                }
            }

            // 隣接ノードを確認
            if let Some(edges) = self.edges.get(&node_id) {
                for edge in edges {
                    let next = State {
                        node_id: edge.node_b_id,
                        cost: cost + edge.weight,
                    };

                    let current_distance =
                        distances.get(&next.node_id).cloned().unwrap_or(i32::MAX);

                    // より短い経路が見つかったら更新
                    if next.cost < current_distance {
                        distances.insert(next.node_id, next.cost);
                        heap.push(next);
                    }
                }
            }
        }

        distances
    }
}

// 2つのノード間の最短距離を求める関数
fn calculate_distance(graph: &Graph, node_id_1: i32, node_id_2: i32) -> i32 {
    let distances_from_node_1 = graph.dijkstra(node_id_1);
    // node_id_2 までの距離を取得し、なければ i32::MAX を返す
    distances_from_node_1
        .get(&node_id_2)
        .cloned()
        .unwrap_or(i32::MAX)
}
