use std::{collections::HashMap, sync::{Arc, Mutex}};
use common::{message::{api::MessageFromApi, engine::{OrderFill, OrderPlacedResponse}}, types::order::{Fill, OrderSide, OrderType}};
use r2d2_redis::{r2d2::{Pool}, redis::Commands, RedisConnectionManager};
use crate::{engine::{AssetBalance, UserAssetBalance}, errors::{BalanceError, EngineError, OrderBookError}, order::{Order, Price}};

pub type RedisResponse = Result<(), r2d2_redis::redis::RedisError>;

const QUOTE:&str = "USDC";

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub base_asset: String,
    pub base_decimals: u8,
    pub market: String,
    pub trade_id: u32,
    pub bids: HashMap<Price,Vec<Order>>,    
    pub asks: HashMap<Price,Vec<Order>>,
    pub last_price: Price,
}

#[derive(Debug)]
pub struct CompleteFill {
    pub order_id: String,
    pub price: Price,
}

impl OrderBook {

    pub fn new(base_asset:String, base_decimals:u8) -> Self {

        let bids = HashMap::new();
        let asks = HashMap::new();

        // ex: SOL_USDC
        let market = format!("{}_{}",base_asset, QUOTE);

        Self { 
            base_asset,
            market, 
            bids,
            asks,
            base_decimals,
            last_price:0,
            trade_id:0,
        }
    }

    pub fn get_base_lamports(&self) -> u64 {
        let base: u32 = 10; 
        let base_decimals = u32::from(self.base_decimals);
        let base_lamports = base.pow(base_decimals) as u64;

        base_lamports
    }

    pub fn validate_and_lock_user_balance(
        &self,
        order:&Order,
        user_balances:&Arc<Mutex<UserAssetBalance>>,
    ) -> Result<(), EngineError>{

        let base_asset = &self.base_asset;
        let quote_asset = QUOTE;        

        let total_price: u64;
        let order_quantity = order.quantity as u64;

        let base_lamports = self.get_base_lamports();

        let mut guard =  user_balances.lock().unwrap();
        let user_asset_balance = guard.get_mut(&order.user_id);

        let asset;

        match user_asset_balance {

            Some(user_balance) => {

                let asset_balance = match order.side {

                    OrderSide::Buy => {

                        asset = quote_asset;
                        
                        total_price = order.price * order_quantity;

                        if user_balance.get(quote_asset).is_none(){
                            user_balance.insert(quote_asset.to_string(), AssetBalance::new());
                        }

                        user_balance.get_mut(quote_asset).unwrap()

                    },
                    OrderSide::Sell => {

                        asset = base_asset;

                        total_price = order_quantity * base_lamports;

                        if user_balance.get(base_asset).is_none() {
                            user_balance.insert(base_asset.to_string(), AssetBalance::new());
                        }

                        user_balance.get_mut(base_asset).unwrap()
                    },
                };

                println!("{} {} balance before lock : {:?}", &order.user_id, &asset, asset_balance);
                println!("amount to lock : {}", total_price);

                if asset_balance.available_amount >= total_price {
                    asset_balance.locked_amount += total_price;
                    asset_balance.available_amount -= total_price;

                    println!("{} {} balance after lock : {:?}", &order.user_id, &asset, asset_balance);
                }
                else{
                    // DROPPING IS NECESSARY, ELSE THIS WONT UNLOCK THE MUTEX
                    drop(guard);
                    println!("user : {} doesnt have enough balance for asset : {:?} ", &order.user_id, &asset);
                    return Err(EngineError::BalanceError(BalanceError::InsufficientBalance));
                }

            },
            None => {
                // DROPPING IS NECESSARY, ELSE THIS WONT UNLOCK THE MUTEX
                drop(guard);
                println!("user : {} not found", order.user_id);
                return Err(EngineError::OrderBookError(OrderBookError::UserNotFound));
            },
        };

        return Ok(());

    }

    pub fn get_desc_bids(&mut self) -> Vec<(&u64, &mut Vec<Order>)>{
        let mut bids:Vec<(&u64, &mut Vec<Order>)> = self.bids.iter_mut().collect();
        bids.sort_by(|a, b| b.0.cmp(a.0));
        bids
    }

    pub fn get_asc_asks(&mut self) -> Vec<(&u64, &mut Vec<Order>)>{

        let mut asks:Vec<(&u64, &mut Vec<Order>)> = self.asks.iter_mut().collect();
        asks.sort_by(|a, b| a.0.cmp(b.0));
        asks
    }

    /// When the orders are completely filled, remove it from the orderbook
    pub fn remove_complete_filled_orders(
        &mut self,
        complete_fill_orders:Vec<CompleteFill>,
        filled_side: OrderSide,
    ){
        let orderside = match filled_side {
            OrderSide::Buy => {
                &mut self.bids
            },
            OrderSide::Sell => {
                &mut self.asks
            }
        };

        for complete_fill_order in complete_fill_orders {

            let price = orderside.get_mut(&complete_fill_order.price);
            
            match price {

                Some(orders) => {
                    let complete_fill_order_id = complete_fill_order.order_id;

                    println!(
                        "removing complete order: {} from book with price :{} on side : {:?}", 
                        complete_fill_order_id, 
                        &complete_fill_order.price,
                        filled_side,
                    );

                    // retain all orders except the complete fill order
                    orders.retain(|o| o.id != complete_fill_order_id);

                    // remove the price entry when there are no orders
                    if orders.len() == 0 {
                        println!("removing price entry : {} from {:?}", &complete_fill_order.price, filled_side);
                        orderside.remove(&complete_fill_order.price);
                    }
                },
                None => {
                    println!("price : {} not found in the orderside : {:?}", &complete_fill_order.price, filled_side);
                }
            }
            
        }

    }

    pub fn match_opposing_orders(
        &mut self,
        order:&Order
    ) -> (u16, Vec<Fill>, Vec<CompleteFill>){

        let mut remaining_quantity = order.quantity;
        
        let mut fill_orders: Vec<Fill> = vec![];
        let mut complete_fill_orders: Vec<CompleteFill>= vec![];

        let mut trade_id = self.trade_id;
        let mut last_price = self.last_price;

        let opposing_side_with_orders = match order.side{

            OrderSide::Buy => {
                let asks: Vec<(&u64, &mut Vec<Order>)> = self.get_asc_asks();
                asks
            },
            OrderSide::Sell => {
                let bids = self.get_desc_bids();
                bids
            }
        };

        for (opposing_price, opposing_orders) in opposing_side_with_orders {

            if opposing_price > &order.price {
                println!("maker price : {} gets higher than the order price : {}", opposing_price, order.price);
                break;
            }

            for opposing_order in opposing_orders {

                if remaining_quantity == 0{
                    break;
                }

                let filled_quantity = opposing_order.quantity.min(remaining_quantity);
                remaining_quantity -= filled_quantity;
                opposing_order.quantity -= filled_quantity;
                
                trade_id+=1;
                last_price = *opposing_price;

                let fill = Fill {
                    maker_id: opposing_order.user_id.clone(),
                    order_id: opposing_order.id.clone(),
                    trade_id,
                    price: *opposing_price,
                    quantity: filled_quantity,
                };   

                println!("matched fill : {:?} for order id: {}", fill, order.id);

                fill_orders.push(fill);

                if opposing_order.quantity == 0 {

                    let complete_fill = CompleteFill {
                        order_id: opposing_order.id.clone(),
                        price: *opposing_price,
                    };

                    complete_fill_orders.push(complete_fill);
                }        
                
            }
        }

        // FINALLY SET THE TRADEID BACK TO ORDERBOOK'S TRADEID
        self.trade_id = trade_id;
        self.last_price = last_price;
    
        if remaining_quantity != order.quantity {
            let filled_quantity = order.quantity - remaining_quantity;
            println!("filled {} quantities of {} for order : {}", filled_quantity, order.quantity, &order.id);
        }
        else{
            println!("filled 0 quantities of {} for order : {}", order.quantity, &order.id)
        }        

        (remaining_quantity, fill_orders, complete_fill_orders)

    }


    pub fn add_order(&mut self, order:Order){

        let price = order.price;

        let orderside = match order.side {
            OrderSide::Buy => {
                &mut self.bids
            },
            OrderSide::Sell => {
                &mut self.asks
            }
        };

        let orders_res = orderside.get_mut(&price);

        match orders_res {
            Some(orders) => {
                println!("addding {:?} in existing price orders on {:?}", &order, &order.side);
                orders.push(order);
            },
            None => {
                println!("addding  {:?} in new price orders on {:?}", &order, &order.side);
                let orders = vec![order];
                orderside.insert(price, orders);
            }
        }

    }   

    pub fn settle_user_balance(
        &self,
        user_id:String,
        order_side:OrderSide,
        filled_orders: &Vec<Fill>,
        user_balances:Arc<Mutex<UserAssetBalance>>,
    ){

        let mut guard = user_balances.lock().unwrap();

        let mut user_base_amount = 0_u64;
        let mut user_quote_amount = 0_u64;

        // settle makers
        for filled_order in filled_orders.iter() {

            let maker_asset_balance = guard.get_mut(&filled_order.maker_id).unwrap();
            
            let quantity = filled_order.quantity as u64;

            // Price will already be in lamports
            let price = filled_order.price;

            let base_lamports = self.get_base_lamports();

            let base_amount_lamports = quantity * base_lamports;
            let quote_amount_lamports = quantity * price;  

            user_base_amount += base_amount_lamports;
            user_quote_amount += quote_amount_lamports;

            match order_side {
                OrderSide::Buy => {
                    // Increment the Quote and Decrement the Base
                    
                    let maker_base_balance = maker_asset_balance.get_mut(&self.base_asset).unwrap();
                    maker_base_balance.locked_amount -= base_amount_lamports;

                    let maker_quote_balance = maker_asset_balance.get_mut(QUOTE).unwrap();
                    maker_quote_balance.available_amount += quote_amount_lamports;
                    
                },
                OrderSide::Sell => {
                    // Increment the Base and decrement the Quote

                    let maker_base_balance = maker_asset_balance.get_mut(&self.base_asset).unwrap();
                    maker_base_balance.available_amount += base_amount_lamports;

                    let maker_quote_balance = maker_asset_balance.get_mut(QUOTE).unwrap();
                    maker_quote_balance.locked_amount -= quote_amount_lamports;
                }
            }
        }

        let user_asset_balance = guard.get_mut(&user_id).unwrap();

        // settle user

        match order_side {
            OrderSide::Buy => {
                // INCREMENT BASE AND DECREMENT QUOTE

                println!("total base amount to add : {} and quote amount to reduce : {}", user_base_amount, user_quote_amount);
                
                let user_base_balance = user_asset_balance.get_mut(&self.base_asset).unwrap();
                println!("{} {} balance before settling : {:?}", &user_id, &self.base_asset, user_base_balance);
                user_base_balance.available_amount += user_base_amount;
                println!("{} {} balance after settling : {:?}", &user_id, &self.base_asset, user_base_balance);

                let user_quote_balance = user_asset_balance.get_mut(QUOTE).unwrap();
                println!("{} {} balance before settling : {:?}", &user_id, QUOTE, user_quote_balance);
                user_quote_balance.locked_amount -= user_quote_amount;
                println!("{} {} balance after settling : {:?}", &user_id, QUOTE, user_quote_balance);

            },
            OrderSide::Sell => {
                // DECREMENT BASE AND INCREMENT QUOTE

                println!("total base amount to reduce : {} and quote amount to add : {}", user_base_amount, user_quote_amount);

                let user_base_balance = user_asset_balance.get_mut(&self.base_asset).unwrap();
                println!("{} {} balance before settling : {:?}", &user_id, &self.base_asset, user_base_balance);
                user_base_balance.locked_amount -= user_base_amount;
                println!("{} {} balance after settling : {:?}", &user_id, &self.base_asset, user_base_balance);

                let user_quote_balance = user_asset_balance.get_mut(QUOTE).unwrap();
                println!("{} {} balance before settling : {:?}", &user_id, QUOTE, user_quote_balance);
                user_quote_balance.available_amount += user_quote_amount;
                println!("{} {} balance after settling : {:?}", &user_id, QUOTE, user_quote_balance);
            }
        }


    }


    pub fn can_place_market_order(&self, order:&Order) -> Result<(), EngineError> {

        let price = order.price;
        let quantities_to_match = order.quantity;

        let orderside = match order.side {
            OrderSide::Buy => {
                &self.asks
            },
            OrderSide::Sell => {
                &self.bids
            }
        };

        match orderside.get(&price) {
            None => {
                // throw error order cannot be matched
                println!("
                    cant place market order : {} as 0 orders available on price : {}",
                    order.id,
                    order.price,
                );

                Err(EngineError::OrderBookError(OrderBookError::ExecuteMarketOrder))
                
            },
            Some(orders) => {

                let mut available_quantities: u16 = 0;

                for order in orders.iter(){
                    let quantity = order.quantity;
                    available_quantities += quantity;
                }

                if available_quantities >= quantities_to_match {
                    Ok(())
                }
                else{
                    println!(
                        "cant place market order :{} as expected: {} available: {} on price : {}",
                        order.id,
                        quantities_to_match, 
                        available_quantities,
                        order.price,
                    );

                    Err(EngineError::OrderBookError(OrderBookError::ExecuteMarketOrder))
                }

            }
        }
    }

    pub fn process_order(
        &mut self, 
        mut order:Order,
        user_balances:Arc<Mutex<UserAssetBalance>>,
    ) -> Result<OrderPlacedResponse, EngineError>{

        /*
            - Check user has enough balance
            - Lock the balance
            - Get the opposiing orders,
            - match the opposing orders till the opposing order price becomes greater than order price
            - after matching settle the user balances
        */

        let maker_side = order.get_opposing_side();

        if order.order_type == OrderType::Market {
            self.can_place_market_order(&order)?;
        }

        self.validate_and_lock_user_balance(&order, &user_balances)?;

        let (
            remaining_quantity,
            filled_orders,
            complete_fill_orders
        ) = self.match_opposing_orders(&order);
        
        self.remove_complete_filled_orders(complete_fill_orders, maker_side);

        let filled_quantity = order.quantity - remaining_quantity;
        // update the order quantity after matching
        order.quantity = remaining_quantity;

        // sit on the orderbook !!
        if remaining_quantity > 0 {
            self.add_order(order.clone());
        }   

        self.settle_user_balance(
            order.user_id, 
            order.side, 
            &filled_orders, 
            user_balances
        );

        let filled_orders:Vec<OrderFill> = filled_orders.iter().map(|o| OrderFill{
            price: o.price,
            quantity: o.quantity,
            trade_id: o.trade_id,
        }).collect();

        let order_placed = OrderPlacedResponse {
            executed_quantity: filled_quantity,
            order_id: order.id,
            fills: filled_orders,
        };

        Ok(order_placed)
    }

    pub fn process(
            &mut self, 
            message_type:MessageFromApi, 
            user_balances:Arc<Mutex<UserAssetBalance>>,
            pool:&Pool<RedisConnectionManager>
    ){
        let mut conn = pool.get().unwrap();

        match message_type {

            MessageFromApi::CreateOrder(payload) => {

                let order = Order::from_create_order_payload(payload);
                let order_id = order.id.clone();

                let res = self.process_order(order, user_balances);
                let error_message = "Error while placing order".to_string();

                match res {
                    Ok(order_placed) => {
                        
                        let serialized = serde_json::to_string(&order_placed);
                        
                        let message;

                        if serialized.is_err(){
                            message = error_message;
                        }
                        else{
                            message = serialized.unwrap();
                        }

                        let redis_response:RedisResponse = conn.publish(&order_id, message);

                        if let Err(e) = redis_response {
                            println!("Error while publishing to the order id : {}", e);
                        }
                    },
                    Err(e) => {
                        println!("Error while executing orders : {:?}", e);

                        let failed_order_placed = OrderPlacedResponse{
                            executed_quantity:0,
                            fills:vec![],
                            order_id:order_id.clone(),
                        };

                        let serialized = serde_json::to_string(&failed_order_placed);

                        let message;
                        if serialized.is_err(){
                            message = error_message;
                        }
                        else{
                            message = serialized.unwrap();
                        }

                        let redis_response:RedisResponse = conn.publish(&order_id, message);

                        if let Err(e) = redis_response {
                            println!("Error while publishing to the order id : {}", e);
                        }
                    }
                }

                
            },
        }

    }

}
