use std::sync::{Arc, Mutex};
use common::message::{api::UserMessageFromApi, engine::{AssetAndBalance, UserBalanceResponse, UserMessageFromEngine}};

use crate::{engine::UserAssetBalance, errors::EngineError, services::redis::RedisService};

pub struct User;

impl User {

    pub fn process_user_message(
        message:String,
        user_balances: Arc<Mutex<UserAssetBalance>>,
        redis_service: &RedisService
    ){
        let try_user_message = UserMessageFromApi::try_deserialized(&message);
        
        match try_user_message {
            Ok(user_message) => {

                let channel;
                let res;

                match user_message {
                    UserMessageFromApi::Balance(user_id) => {
                        channel = format!("{}-balance", user_id);
                        res = User::get_user_asset_balance(user_id, user_balances);
                    }
                }

                redis_service.publish_user_message_to_api(channel, res);

            },
            Err(e) => {
                println!("error : {e} while deserializing user message");
            }
        }
    }

    pub fn get_user_asset_balance(
        user_id:String, 
        user_balances: Arc<Mutex<UserAssetBalance>>
    ) -> Result<UserMessageFromEngine, EngineError>{

        let guard = user_balances.lock().unwrap();
        let try_assets_balance = guard.get(&user_id);

        match try_assets_balance {

            Some(assets_balance) => {

                let mut balances = vec![];

                for (asset, balance) in assets_balance {
                    balances.push(
                        AssetAndBalance {
                            asset: asset.to_owned(),
                            balance: balance.available_amount
                        }
                    );
                }

                let user_balance_res = UserBalanceResponse {
                    user_id,
                    balances,
                };

                let user_message = UserMessageFromEngine::Balance(user_balance_res);

                Ok(user_message)
            },
            None => {
                Err(EngineError::UserNotFound)
            }
        }
    }


}