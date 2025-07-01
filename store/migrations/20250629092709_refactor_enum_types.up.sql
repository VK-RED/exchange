-- Add up migration script here
ALTER TABLE "order" ALTER COLUMN "side" TYPE VARCHAR(255);
ALTER TABLE "order" ALTER COLUMN "order_status" TYPE VARCHAR(255);