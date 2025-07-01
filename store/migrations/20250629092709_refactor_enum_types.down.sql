-- Add down migration script here
CREATE TYPE OrderSide IF NOT EXISTS as ENUM ('Buy', 'Sell');
CREATE TYPE OrderStatus IF NOT EXISTS as ENUM ('Open', 'Filled', 'Cancelled');

ALTER TABLE "order" ALTER COLUMN "side" OrderSide NOT NULL;
ALTER TABLE "order" ALTER COLUMN "order_status" OrderStatus NOT NULL;

