-- このファイルに記述されたSQLコマンドが、マイグレーション時に実行されます。
ALTER TABLE users ADD INDEX index_users_on_username(username);

ALTER TABLE orders ADD COLUMN area_id INT NULL;

UPDATE orders
SET area_id = (
    SELECT area_id
    FROM nodes
    WHERE nodes.id = orders.node_id
);

ALTER TABLE orders
ADD INDEX idx_orders_area_status (area_id, status);