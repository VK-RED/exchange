-- Add up migration script here
CREATE TYPE OrderSide as ENUM ('Buy', 'Sell');
CREATE TYPE OrderStatus as ENUM ('Open', 'Filled', 'Cancelled');

CREATE TABLE IF NOT EXISTS "order"(
    id VARCHAR(255) PRIMARY KEY NOT NULL,
    quantity VARCHAR(255) NOT NULL,
    filled_quantity VARCHAR(255) NOT NULL,
    price VARCHAR(255) NOT NULL,
    side OrderSide NOT NULL,
    order_status OrderStatus NOT NULL
);

CREATE TABLE IF NOT EXISTS "trade"(
    id bigint PRIMARY KEY NOT NULL,
    matched_at bigint NOT NULL,
    market VARCHAR(255) NOT NULL,
    price VARCHAR(255) NOT NULL,
    quantity VARCHAR(255) NOT NULL,
    quote_qty VARCHAR(255) NOT NULL
);


