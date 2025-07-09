use serde::{Deserialize, Serialize};
use sqlx::{Error, Pool, Postgres};

#[derive(Debug, Deserialize, Serialize)]
pub struct Order{
    pub id: String,
    pub quantity: String, 
    pub filled_quantity: String,
    pub price: String,
    pub side: String,
    pub order_status: String,
}

pub struct UpdateDbOrder{
    pub order_id: String,
    pub filled_quantity: String,
    pub status: String,
}

impl Order {

    pub async fn add_order(
        order:Order, 
        pool:&Pool<Postgres>
    ) -> Result<Order,Error>{
        
        let order =  sqlx::query_as!(
            Order,
            r#"
                INSERT INTO "order" (id, quantity, filled_quantity, price, side, order_status)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *
            "#,
            order.id,
            order.quantity,
            order.filled_quantity,
            order.price,
            order.side,
            order.order_status,
        )
        .fetch_one(pool)
        .await?;

        Ok(order)
                
    }

    pub async fn update_orders(orders:Vec<UpdateDbOrder>, pool:&Pool<Postgres>) -> Result<(), Error>{

        if orders.len() == 0{
            return Ok(());
        }

        let mut results = vec![];

        for order in orders {

            let res =  sqlx::query!(
                r#"
                    UPDATE "order"
                    SET filled_quantity = $1, order_status = $2
                    WHERE id = $3;  
                "#,
                order.filled_quantity,
                order.status,
                order.order_id
            )
            .execute(pool);

            results.push(res);
        }
        
        for res in results {
            res.await?;
        }

        Ok(())

    }

    pub async fn update_cancelled_orders(orders:Vec<String>, pool:&Pool<Postgres>) -> Result<(), Error>{

        if orders.len() == 0 {
            println!("no orders to cancel !");
            return Ok(());
        }

        let mut query_builder: sqlx::QueryBuilder<'_, Postgres> = sqlx::QueryBuilder::new(r#"
            UPDATE "order"
            SET order_status = 'Cancelled'
            WHERE id in (
        "#);

        let mut separated = query_builder.separated(", ");

        for order in orders {
            separated.push_bind(order);
        }

        separated.push_unseparated(")");

        query_builder.build().execute(pool).await?;

        Ok(())
    }

}