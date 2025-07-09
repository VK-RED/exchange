use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}};
use chrono::Utc;
use common::{message::{api::{CancelOrderPayload, MessageFromApi}, db_filler::{AddOrderToDb, OrderStatus, Trade, UpdateOrder}, engine::{CancelAllOrders, DepthResponse, MessageFromEngine, OpenOrder, OrderCancelledResponse, OrderFill, OrderPlacedResponse}}, types::order::{Fill, OrderSide, OrderType, Price, Quantity}};
use rust_decimal::{dec, Decimal, prelude::ToPrimitive};
use crate::{engine::{AssetBalance, UserAssetBalance}, errors::{EngineError}, order::{Order, OrdersWithQuantity}, services::redis::RedisService};

const QUOTE:&str = "USDC";
const QUOTE_LAMPORTS:u64 = 1000_000;

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub base_asset: String,
    pub base_decimals: u8,
    pub market: String,
    pub trade_id: u32,
    pub bids: HashMap<Price,OrdersWithQuantity>,    
    pub asks: HashMap<Price,OrdersWithQuantity>,
    pub last_price: Price,
}

#[derive(Debug)]
pub struct CompleteFill {
    pub order_id: String,
    pub price: Price,
}

#[derive(Debug)]
pub struct PriceWithDepth{
    pub updated_bids: HashMap<Price,Quantity>,
    pub updated_asks: HashMap<Price,Quantity>,
}

impl PriceWithDepth{
    pub fn new() -> Self{
        Self {
            updated_asks: HashMap::new(),
            updated_bids: HashMap::new(),
        }
    }

    pub fn update_depth(&mut self, side: OrderSide, price: Price, quantity:Quantity){
        match side{
            OrderSide::Buy => {
                self.updated_bids.insert(price, quantity);
            },
            OrderSide::Sell => {
                self.updated_asks.insert(price, quantity);
            }
        }
    }
    
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
            last_price:dec!(0),
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

        let mut total_price;
        let order_quantity = order.quantity;

        let base_lamports = self.get_base_lamports();

        let mut guard =  user_balances.lock().unwrap();
        let user_asset_balance = guard.get_mut(&order.user_id);

        let asset;

        match user_asset_balance {

            Some(user_balance) => {

                let asset_balance = match order.side {

                    OrderSide::Buy => {

                        asset = quote_asset;
                        
                        // consider this
                        // total_price = 150.02  * 2.5 * 1000_000
                        // total_price = 375050000.000
                        total_price = order.price * order_quantity * Decimal::from(QUOTE_LAMPORTS);

                        // the total_price can become a decimal even after multiplying with lamports
                        // so use trunc() to cut the extra decimals
                        // ex: 150234567.43435345 => 150234567

                        total_price = total_price.trunc();

                        if user_balance.get(quote_asset).is_none(){
                            user_balance.insert(quote_asset.to_string(), AssetBalance::new());
                        }

                        user_balance.get_mut(quote_asset).unwrap()

                    },
                    OrderSide::Sell => {

                        asset = base_asset;

                        // consider this
                        // total_price = 150.02  * 1000_000
                        // total_price = 150020000.000

                        total_price = order_quantity * Decimal::from(base_lamports);

                        // the total_price can become a decimal even after multiplying with lamports
                        // so use trunc() to cut the extra decimals
                        // ex: 15023456723.43435345 => 15023456723
                        total_price = total_price.trunc();

                        if user_balance.get(base_asset).is_none() {
                            user_balance.insert(base_asset.to_string(), AssetBalance::new());
                        }

                        user_balance.get_mut(base_asset).unwrap()
                    },
                };

                let total_amount = total_price.to_u64().expect("None while converting total_price in locking user balance");

                println!("{} {} balance before lock : {:?}", &order.user_id, &asset, asset_balance);
                println!("amount to lock : {}", total_amount);

                if asset_balance.available_amount >= total_amount {
                    asset_balance.locked_amount += total_amount;
                    asset_balance.available_amount -= total_amount;

                    println!("{} {} balance after lock : {:?}", &order.user_id, &asset, asset_balance);
                }
                else{
                    // DROPPING IS NECESSARY, ELSE THIS WONT UNLOCK THE MUTEX
                    drop(guard);
                    println!("user : {} doesnt have enough balance for asset : {:?} ", &order.user_id, &asset);
                    return Err(EngineError::InsufficientBalance);
                }

            },
            None => {
                // DROPPING IS NECESSARY, ELSE THIS WONT UNLOCK THE MUTEX
                drop(guard);
                println!("user : {} not found", order.user_id);
                return Err(EngineError::UserNotFound);
            },
        };

        return Ok(());

    }

    pub fn get_desc_bids(&mut self) -> Vec<(&Decimal, &mut OrdersWithQuantity)>{
        let mut bids:Vec<(&Decimal, &mut OrdersWithQuantity)> = self.bids.iter_mut().collect();
        bids.sort_by(|a, b| b.0.cmp(a.0));
        bids
    }

    pub fn get_asc_asks(&mut self) -> Vec<(&Decimal, &mut OrdersWithQuantity)>{

        let mut asks:Vec<(&Decimal, &mut OrdersWithQuantity)> = self.asks.iter_mut().collect();
        asks.sort_by(|a, b| a.0.cmp(b.0));
        asks
    }

    /// When the orders are completely filled, remove it from the orderbook
    pub fn remove_complete_filled_orders(
        &mut self,
        complete_fill_orders:Vec<CompleteFill>,
        filled_side: OrderSide,
    ){
        let price_w_orders_n_quantity = match filled_side {
            OrderSide::Buy => {
                &mut self.bids
            },
            OrderSide::Sell => {
                &mut self.asks
            }
        };

        for complete_fill_order in complete_fill_orders {

            let orders_and_quantity = price_w_orders_n_quantity.get_mut(&complete_fill_order.price);
            
            match orders_and_quantity {

                Some(orders_and_quantity) => {

                    let orders = &mut orders_and_quantity.orders;

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
                        price_w_orders_n_quantity.remove(&complete_fill_order.price);
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
        order:&mut Order,
        price_w_updated_depths: &mut PriceWithDepth,
    ) -> (Vec<Fill>, Vec<CompleteFill>){

        let mut remaining_quantity = order.quantity;
        
        let mut fill_orders: Vec<Fill> = vec![];
        let mut complete_fill_orders: Vec<CompleteFill>= vec![];

        let mut trade_id = self.trade_id;
        let mut last_price = self.last_price;

        let opposing_side_with_orders = match order.side{

            OrderSide::Buy => {
                let asks = self.get_asc_asks();
                asks
            },
            OrderSide::Sell => {
                let bids = self.get_desc_bids();
                bids
            }
        };

        for (opposing_price, orders_with_quantity) in opposing_side_with_orders {

            let can_match_orders = match order.side {
                OrderSide::Buy => {
                    opposing_price <= &order.price
                },
                OrderSide::Sell => {
                    opposing_price >= &order.price
                }
            };

            if !can_match_orders {
                println!("cant match maker price : {} against order price : {}", opposing_price, order.price);
                break;
            }

            let opposing_orders = orders_with_quantity.orders.iter_mut();
            let mut opposing_total_quantity = &mut orders_with_quantity.total_quantity;

            for opposing_order in opposing_orders {

                if remaining_quantity == dec!(0){
                    break;
                }

                let filled_quantity = (opposing_order.quantity - opposing_order.filled).min(remaining_quantity);
                
                remaining_quantity -= filled_quantity;

                opposing_order.filled += filled_quantity;
                order.filled += filled_quantity;

                opposing_total_quantity -= filled_quantity;

                let opposing_side= order.get_opposing_side();

                price_w_updated_depths.update_depth(opposing_side,*opposing_price, *opposing_total_quantity);
                
                trade_id+=1;
                last_price = *opposing_price;

                let fill = Fill {
                    maker_id: opposing_order.user_id.clone(),
                    order_id: opposing_order.id.clone(),
                    trade_id,
                    price: *opposing_price,
                    quantity: opposing_order.quantity,
                    filled_quantity: filled_quantity,
                };   

                println!("matched fill : {:?} for order id: {}", fill, order.id);

                fill_orders.push(fill);

                if opposing_order.quantity == opposing_order.filled {

                    let complete_fill = CompleteFill {
                        order_id: opposing_order.id.clone(),
                        price: *opposing_price,
                    };

                    complete_fill_orders.push(complete_fill);
                }        
                
            }
        }

        // FINALLY SET THE TRADEID AND LAST PRICE BACK TO ORDERBOOK'S TRADEID
        self.trade_id = trade_id;
        self.last_price = last_price;
    
        if remaining_quantity != order.quantity {
            let filled_quantity = order.quantity - remaining_quantity;
            println!("filled {} quantities of {} for order : {}", filled_quantity, order.quantity, &order.id);
        }
        else{
            println!("filled 0 quantities of {} for order : {}", order.quantity, &order.id)
        }        

        (fill_orders, complete_fill_orders)

    }


    pub fn add_order(
        &mut self, 
        order:Order,
        price_w_updated_depths: &mut PriceWithDepth,
    ){

        let price = order.price;
        let order_side = order.side;

        let price_w_orders_n_qty = match order.side {
            OrderSide::Buy => {
                &mut self.bids
            },
            OrderSide::Sell => {
                &mut self.asks
            }
        };

        let orders_w_qty_res = price_w_orders_n_qty.get_mut(&price);

        match orders_w_qty_res {
            Some(orders_w_quantity) => {
                println!("addding {:?} in existing price orders on {:?}", &order, &order.side);
                let order_qty = order.quantity - order.filled;
                orders_w_quantity.orders.push(order);
                orders_w_quantity.total_quantity += order_qty;

                let total_quantity = orders_w_quantity.total_quantity;

                price_w_updated_depths.update_depth(order_side,price,total_quantity);
            },
            None => {
                println!("addding  {:?} in new price orders on {:?}", &order, &order.side);
                let remaining_qty = order.quantity - order.filled;
                let orders = vec![order];
                let orders_w_qty = OrdersWithQuantity::new(remaining_qty, orders);
                price_w_updated_depths.update_depth(order_side,price, remaining_qty);
                price_w_orders_n_qty.insert(price, orders_w_qty);
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
            
            let quantity = filled_order.filled_quantity;
            let price = filled_order.price;

            let base_lamports = self.get_base_lamports();

            let base_amount_in_lamports = quantity * Decimal::from(base_lamports);
            let quote_amount_in_lamports = quantity * price * Decimal::from(QUOTE_LAMPORTS);  

            // convert to u64 
            let base_amount_in_lamports = base_amount_in_lamports
            .to_u64()
            .expect("None while converting base amount in settling user balance");

            let quote_amount_in_lamports = quote_amount_in_lamports
            .to_u64()
            .expect("None while converting qupte amount in settling user balance");

            user_base_amount += base_amount_in_lamports;
            user_quote_amount += quote_amount_in_lamports;

            match order_side {
                OrderSide::Buy => {
                    // Increment the Quote and Decrement the Base
                    
                    let maker_base_balance = maker_asset_balance.get_mut(&self.base_asset).unwrap();
                    maker_base_balance.locked_amount -= base_amount_in_lamports;

                    let maker_quote_balance = maker_asset_balance.get_mut(QUOTE).unwrap();
                    maker_quote_balance.available_amount += quote_amount_in_lamports;
                    
                },
                OrderSide::Sell => {
                    // Increment the Base and decrement the Quote

                    let maker_base_balance = maker_asset_balance.get_mut(&self.base_asset).unwrap();
                    maker_base_balance.available_amount += base_amount_in_lamports;

                    let maker_quote_balance = maker_asset_balance.get_mut(QUOTE).unwrap();
                    maker_quote_balance.locked_amount -= quote_amount_in_lamports;
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

        let price_w_orders_n_qty = match order.side {
            OrderSide::Buy => {
                &self.asks
            },
            OrderSide::Sell => {
                &self.bids
            }
        };

        match price_w_orders_n_qty.get(&price) {
            None => {
                // throw error order cannot be matched
                println!("
                    cant place market order : {} as 0 orders available on price : {}",
                    order.id,
                    order.price,
                );

                Err(EngineError::PartialOrderFill)
                
            },
            Some(orders_w_qty) => {

                let available_quantities = orders_w_qty.total_quantity;

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

                    Err(EngineError::PartialOrderFill)
                }

            }
        }
    }

    pub fn process_order(
        &mut self, 
        order:&mut Order,
        user_balances:Arc<Mutex<UserAssetBalance>>,
    ) -> Result<(OrderPlacedResponse, PriceWithDepth), EngineError>{

        /*
            - Check user has enough balance
            - Lock the balance
            - Get the opposiing orders,
            - match the opposing orders till the opposing order price becomes greater than order price
            - after matching settle the user balances
        */

        // Truncate price and quantity 
        order.price = order.price.trunc_with_scale(9);
        order.quantity = order.quantity.trunc_with_scale(6);

        let maker_side = order.get_opposing_side();

        let mut price_w_depth = PriceWithDepth::new();

        if order.order_type == OrderType::Market {
            self.can_place_market_order(order)?;
        }

        self.validate_and_lock_user_balance(order, &user_balances)?;

        let (
            filled_orders,
            complete_fill_orders
        ) = self.match_opposing_orders(order, &mut price_w_depth);
        
        self.remove_complete_filled_orders(complete_fill_orders, maker_side);

        // sit on the orderbook !!
        if order.filled < order.quantity {
            self.add_order(order.clone(), &mut price_w_depth);
        }   

        self.settle_user_balance(
            order.user_id.clone(), 
            order.side, 
            &filled_orders, 
            user_balances
        );

        let filled_orders:Vec<OrderFill> = filled_orders.iter().map(|o| OrderFill{
            order_id: o.order_id.clone(),
            price: o.price,
            quantity: o.quantity,
            filled_quantity: o.filled_quantity,
            trade_id: o.trade_id,
        }).collect();

        let order_placed = OrderPlacedResponse {
            executed_quantity: order.filled,
            order_id: order.id.clone(),
            fills: filled_orders,
        };

        Ok((order_placed, price_w_depth))
    }

    pub fn settle_balance_after_cancel(
        &self, 
        user_id: &String,
        cancelled_order: &OrderCancelledResponse,
        cancelled_price: Decimal,
        user_balances:Arc<Mutex<UserAssetBalance>>,
    ) -> Result<(), EngineError>{

        let mut guard = user_balances.lock().unwrap();
        
        match guard.get_mut(user_id) {
            None => {
                println!("Error while getting guard lock in settling balance after cancel");
                drop(guard);
                Err(EngineError::UserNotFound)
            },
            Some(user_w_asset_balance) => {

                let remaining_qty = cancelled_order.quantity - cancelled_order.executed_quantity;

                let total_amount;
                let asset;

                match cancelled_order.side {
                    OrderSide::Buy => {
                        asset = QUOTE;
                        println!("remaining : {remaining_qty} , cancelled_price:{cancelled_price}");
                        total_amount = (remaining_qty * cancelled_price * Decimal::from(QUOTE_LAMPORTS)).to_u64();
                    },
                    OrderSide::Sell => {
                        asset = &self.base_asset;
                        let base_lamports = self.get_base_lamports();
                        total_amount = (remaining_qty * Decimal::from(base_lamports)).to_u64();
                    }
                }

                match total_amount {
                    Some(amount) => {

                        let err_msg = format!("{} balance not found for user : {}", asset, user_id);

                        println!("asset : {}", asset);
                        let asset_balance = user_w_asset_balance.get_mut(asset).expect(&err_msg);
                        println!("asset_balance : {:?}", asset_balance);
                        println!("amount : {}", amount);
                        asset_balance.locked_amount -= amount;
                        asset_balance.available_amount += amount;

                        Ok(())
                    },
                    None => {
                        println!("Error while settling :{} balance after cancelling, total amount: {:?}", asset, total_amount);
                        drop(guard);
                        Err(EngineError::InternalError)
                    }
                }
            }
        }

    }

    /// returns cancelled_order with price
    pub fn cancel_order_in_side(
        &mut self, 
        side:OrderSide,
        target_order_id: &String,
        user_id:&String,
        price_w_updated_depths: &mut PriceWithDepth
    ) -> Result<Option<(OrderCancelledResponse, Decimal)>, EngineError>{    

        let price_w_orders_n_qty = match side{
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        };

        let mut final_res = None;

        for (price, orders_w_qty) in price_w_orders_n_qty.iter_mut(){

            let mut target_order_index = -1;

            //iterate through orders of the price range
            for (index, order) in orders_w_qty.orders.iter().enumerate() {
                if order.id == *target_order_id {
                    target_order_index = index as i32;
                    break;
                }
            }

            // if found cancel it!!
            if target_order_index != -1 {

                let target_order_index = target_order_index as usize;

                match orders_w_qty.orders.get(target_order_index) {

                    Some(target_order) => {

                        if target_order.user_id != *user_id {
                            println!("{} cannot cancel the order : {:?}", user_id, target_order);
                            return Err(EngineError::MismatchUser);                            
                        }

                        let order = orders_w_qty.orders.remove(target_order_index);
                        let unfilled_qty = order.quantity - order.filled;

                        println!("No of Quantities: {} in price : {} on  {:?} before updating", orders_w_qty.total_quantity, price, side);

                        orders_w_qty.total_quantity -= unfilled_qty;

                        let total_quantity = orders_w_qty.total_quantity;

                        price_w_updated_depths.update_depth(side, *price, total_quantity);

                        println!("No of Quantities: {} in price : {} on  {:?} after updating", orders_w_qty.total_quantity, price, side);

                        let order_cancelled = OrderCancelledResponse {
                            order_id: order.id,
                            quantity: order.quantity,
                            executed_quantity: order.filled,
                            side
                        };

                        println!("cancelled order : {:?}", order_cancelled);
                        final_res = Some((order_cancelled, *price));
                        break;
                    },
                    None =>{
                        println!("Cannnot find target order to cancel in orders at index : {}", target_order_index);
                        return Err(EngineError::InternalError);
                    }
                }
            }
        }

        match final_res {
            Some(res) => {

                let price = res.1;

                // if there are no orders available the clearup the price and orders

                if let Some(orders_w_qty) = price_w_orders_n_qty.get(&price) {
                    if orders_w_qty.orders.len() == 0 {
                        price_w_orders_n_qty.remove(&price);                        
                    }
                }

                Ok(Some(res))
            },
            None => Ok(None)
        }

    }   

    pub fn cancel_order(
        &mut self,
        order_payload:CancelOrderPayload,
        user_balances:Arc<Mutex<UserAssetBalance>>,
    ) -> Result<(OrderCancelledResponse, PriceWithDepth), EngineError>{
        let mut order_cancelled_res;

        let mut price_w_updated_depth = PriceWithDepth::new();

        order_cancelled_res = self.cancel_order_in_side(
            OrderSide::Buy, 
            &order_payload.order_id,
            &order_payload.user_id,
            &mut price_w_updated_depth
        )?;

        if order_cancelled_res.is_none() {
            order_cancelled_res = self.cancel_order_in_side(
                OrderSide::Sell, 
                &order_payload.order_id,
                &order_payload.user_id,
                &mut price_w_updated_depth
            )?;
        }

        match order_cancelled_res {
            Some((res, price)) => {
                self.settle_balance_after_cancel(
                    &order_payload.user_id,
                    &res,
                    price,
                    user_balances
                )?;
                Ok((res, price_w_updated_depth))
            },
            None => {
                Err(EngineError::InvalidOrderId)
            }
        }
    
    }


    fn cancel_all_orders_on_side(
        &mut self,
        user_id: &str,
        side: OrderSide,
        cancelled_orders: &mut Vec<CancelAllOrders>,
        price_w_updated_depths: &mut PriceWithDepth
    ){
        let price_w_orders_n_qty = match side{
            OrderSide::Buy =>{
                &mut self.bids
            },
            OrderSide::Sell=> {
                &mut self.asks
            }
        };

        let mut price_with_empty_orders = vec![];

        // iterate through all the price range and find the orders to remove
        // using iter as to aboid value getting consumed
        for (price, orders_w_qty) in price_w_orders_n_qty.iter_mut() {

            let mut orders_to_remove = HashSet::new();

            // using iter as to avoid orders getting consumed
            for order in orders_w_qty.orders.iter() {
                if order.user_id == user_id {
                    orders_to_remove.insert(order.id.clone());
                }
            }

            orders_w_qty.orders.retain(|order| {

                if orders_to_remove.contains(&order.id) {

                    let unfilled_qty = order.quantity - order.filled;
                    orders_w_qty.total_quantity -= unfilled_qty;

                    println!("order cancelled: {:?}", order);

                    cancelled_orders.push(
                        CancelAllOrders { 
                            order_id: order.id.clone(), 
                            quantity: order.quantity, 
                            executed_quantity:order.filled, 
                            side:order.side, 
                            price: order.price,
                        }
                    );

                    return false;
                }
                return true;
            });

            let total_quantity = orders_w_qty.total_quantity;

            // update depth only if orders has removed on the price range
            if orders_to_remove.len() > 0 {
                price_w_updated_depths.update_depth(side, *price, total_quantity);
            }

            // if there are no orders, then remove the price
            
            if orders_w_qty.orders.len() == 0 {
                println!("no orders left in price : {} on : {:?}", price, side);
                price_with_empty_orders.push(price.clone());
            }

        }

        for price in price_with_empty_orders {
            price_w_orders_n_qty.remove(&price);
        }
        

    }

    pub fn cancel_all_orders(
        &mut self,
        user_id: &str,
        user_balances:Arc<Mutex<UserAssetBalance>>,
    )-> Result<(Vec<CancelAllOrders>, PriceWithDepth), EngineError>{

        let mut cancelled_orders: Vec<CancelAllOrders> = vec![];

        let mut price_w_depths = PriceWithDepth::new();

        self.cancel_all_orders_on_side(
            user_id, 
            OrderSide::Buy, 
            &mut cancelled_orders,
            &mut price_w_depths,
        );

        self.cancel_all_orders_on_side(
            user_id, 
            OrderSide::Sell, 
            &mut cancelled_orders,
            &mut price_w_depths,
        );

        let mut guard = user_balances.lock().unwrap();

        let user_asset_balance = guard.get_mut(user_id).ok_or_else(||{
            EngineError::UserNotFound
        })?;

        for order in  cancelled_orders.iter() {

            let asset;
            let amount;

            match order.side {
                OrderSide::Buy => {
                    asset = QUOTE;
                    let remaining = order.quantity - order.executed_quantity;
                    amount = order.price * remaining * Decimal::from(QUOTE_LAMPORTS);
                },
                OrderSide::Sell => {
                    asset = &self.base_asset;
                    let remaining = order.quantity - order.executed_quantity;
                    amount = remaining * Decimal::from(self.get_base_lamports());
                }
            }

            let total_amount = amount.to_u64().ok_or_else(||{
                println!("none while converting amount: {}", amount);
                EngineError::InternalError
            })?;

            println!("settling {} {} to {}", total_amount, asset, user_id);

            let asset_balance = user_asset_balance.get_mut(asset).ok_or_else(||{
                println!("assetbalance {:?} not set for user : {}", asset, user_id);
                EngineError::InternalError
            })?;

            asset_balance.locked_amount -= total_amount;
            asset_balance.available_amount += total_amount;
        }


        Ok((cancelled_orders, price_w_depths))

    }

    fn get_open_orders_on_side(
        &self,
        user_id: &str,
        side: OrderSide,
        open_orders: &mut Vec<OpenOrder>
    ){
        let price_w_orders_n_qty = match side {
            OrderSide::Buy => &self.bids,
            OrderSide::Sell => &self.asks,
        };

        for (_price, orders_w_qty) in price_w_orders_n_qty {
            for order in orders_w_qty.orders.iter() {
                if order.user_id == user_id {
                    let open_order = OpenOrder {
                        executed_quantity: order.filled,
                        order_id: order.id.clone(),
                        price: order.price,
                        quantity: order.quantity,
                        side: order.side,
                    };
                    open_orders.push(open_order);
                }
            }
        }
        
    }

    pub fn get_all_open_orders(
        &self,
        user_id: &str,
    ) -> Result<Vec<OpenOrder>, EngineError> {

        let mut open_orders = vec![];

        self.get_open_orders_on_side(
            user_id, 
            OrderSide::Buy, 
            &mut open_orders
        );

        self.get_open_orders_on_side(
            user_id, 
            OrderSide::Sell, 
            &mut open_orders
        );
        
        Ok(open_orders)
    }

    pub fn get_depth_on_side(
        &self,
        side:OrderSide,
    ) -> Vec<[Decimal; 2]>{
        let mut price_n_qty = vec![];
        
        let price_w_orders_n_qty = match side {
            OrderSide::Buy => &self.bids,
            OrderSide::Sell => &self.asks,
        };

        for (price, orders_w_qty) in price_w_orders_n_qty {
            let arr = [*price, orders_w_qty.total_quantity];
            price_n_qty.push(arr);
        }

        price_n_qty

    }

    pub fn get_depth(&self) -> Result<DepthResponse,EngineError>{
        let bid_depth = self.get_depth_on_side(OrderSide::Buy);
        let ask_depth = self.get_depth_on_side(OrderSide::Sell);

        let depth = DepthResponse {
            asks: ask_depth,
            bids: bid_depth
        };

        Ok(depth)
    }
   
    pub fn process(
            &mut self, 
            message_type:MessageFromApi, 
            user_balances:Arc<Mutex<UserAssetBalance>>,
            redis:&RedisService,
    ){
        let publish_on_channel;
    
        match message_type {

            MessageFromApi::CreateOrder(payload) => {

                let mut order = Order::from_create_order_payload(payload);

                let order_id = order.id.clone();
                publish_on_channel = &order_id;

                let res = self.process_order(&mut order, user_balances);

                let mut order_to_add = None;
                let mut orders_to_update:Vec<UpdateOrder> = vec![];

                let mut trades = vec![];
                let mut price_w_depth_to_update = None;

                let message = match res {

                    Ok((order_placed, price_w_depth)) => {

                        price_w_depth_to_update = Some(price_w_depth);

                        let order_status = if order.quantity == order_placed.executed_quantity {OrderStatus::Filled} else {OrderStatus::Open};

                        let add_order = AddOrderToDb {
                            filled_quantity: order_placed.executed_quantity,
                            order_id: order.id.clone(),
                            price: order.price,
                            quantity: order.quantity,
                            side: order.side,
                            status: order_status,
                        };

                        order_to_add = Some(add_order);

                        orders_to_update = order_placed.fills.iter().map(|order|{

                            let order_status;

                            match order.filled_quantity == order.quantity {
                                true => order_status = OrderStatus::Filled,
                                false => order_status = OrderStatus::Open,
                            }

                            UpdateOrder {
                                filled_quantity: order.filled_quantity,
                                order_id: order.order_id.clone(),
                                status:order_status,
                            }
                        }).collect();

                        for fill in order_placed.fills.iter(){

                            let now = Utc::now();

                            let trade = Trade {
                                id: fill.trade_id,
                                price: fill.price,
                                quantity: fill.filled_quantity,
                                quote_qty: (order.price * fill.filled_quantity).trunc_with_scale(6),
                                market: self.market.clone(),
                                timestamp: now.timestamp_millis(),
                            };

                            trades.push(trade);
                        }

                        Ok(MessageFromEngine::OrderPlaced(order_placed))
                    },
                    Err(e) => {
                        Err(e)
                    }
                };

                redis.publish_message_to_api(publish_on_channel, message);
                redis.publish_ws_trade(&order.market, &trades);
                redis.publish_ws_depth(&order.market, price_w_depth_to_update);
                redis.publish_trades_to_db(trades);
                redis.update_db_orders(order_to_add, orders_to_update);
                
            },

            MessageFromApi::CancelOrder(order_payload) => {
                let order_id = order_payload.order_id.clone();
                publish_on_channel = &order_id;

                let market = order_payload.market.clone();
                let cancel_order_res = self.cancel_order(order_payload, user_balances);

                let mut updated_depths = None;

                let message = match cancel_order_res {
                    Ok((order_cancel, price_w_depth)) => {
                        updated_depths = Some(price_w_depth);
                        Ok(MessageFromEngine::OrderCancelled(order_cancel))
                    },
                    Err(e) => {
                        Err(e)
                    }
                };

                redis.publish_message_to_api(publish_on_channel, message);
                redis.publish_ws_depth(&market, updated_depths);
                redis.publish_cancel_order_updates(vec![order_id]);
            },

            MessageFromApi::CancelAllOrders(payload) => {

                publish_on_channel = &payload.user_id;
                let market = &payload.market;
                
                let mut updated_depths = None;
                let cancel_all_orders_res = self.cancel_all_orders(&payload.user_id, user_balances);

                let mut cancelled_orders = vec![];

                let message = cancel_all_orders_res.
                map(|(orders, price_w_depth)|{

                    updated_depths = Some(price_w_depth);

                    for order in orders.iter() {
                        cancelled_orders.push(order.order_id.clone());
                    }

                    MessageFromEngine::AllOrdersCancelled(orders)
                } );

                redis.publish_message_to_api(publish_on_channel, message);
                redis.publish_ws_depth(market, updated_depths);
                redis.publish_cancel_order_updates(cancelled_orders);
            },

            MessageFromApi::GetAllOpenOrders(payload) => {

                publish_on_channel = &payload.user_id;

                let get_all_orders_res = self.get_all_open_orders(&payload.user_id);

                let message = get_all_orders_res.
                map(|orders| MessageFromEngine::AllOpenOrders(orders));

                redis.publish_message_to_api(publish_on_channel, message);
            },

            MessageFromApi::GetDepth(market) => {
                publish_on_channel = &market;

                let depth_res = self.get_depth();

                let message = depth_res.map(|depth| MessageFromEngine::GetDepth(depth));
                
                redis.publish_message_to_api(publish_on_channel, message);
            }
        };


    }

}
