use serde::{Deserialize, Serialize};
use sqlx::{Error, Pool, Postgres};

#[derive(Debug, Deserialize, Serialize)]
pub struct Trade {
    pub id: i64,
    pub market: String,
    pub price: String,
    pub quantity: String,
    pub quote_qty: String,
    pub matched_at: i64,
}


impl Trade {

    pub async fn add_trades(trades:Vec<Trade>, pool:&Pool<Postgres>) -> Result<(), Error>{

        // NOTE: QUERY BUILDER WILL THROW ERROR IF THE TRADES ARE EMPTY
        // SO RETURN EALRY IF TRADES IS EMPTY

        if trades.len() == 0{
            return Ok(());
        }

        let mut query_builder: sqlx::QueryBuilder<'_, Postgres> = sqlx::QueryBuilder::new(r#"
            INSERT INTO "trade" (id, market, price, quantity, quote_qty, matched_at)
        "#);

        query_builder.push_values(trades, |mut b, trade|{
            
            // convert to i64
            let id = i64::from(trade.id);
            let matched_at = i64::from(trade.matched_at);

            b.push_bind(id);
            b.push_bind(trade.market);
            b.push_bind(trade.price);
            b.push_bind(trade.quantity);
            b.push_bind(trade.quote_qty);
            b.push_bind(matched_at);
        });
        
        query_builder.build().execute(pool).await?;

        Ok(())
    }


}