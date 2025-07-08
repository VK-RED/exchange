use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::UnboundedSender;
pub type Tx = UnboundedSender<String>;
pub struct UserManager {
    users: HashMap<String, Tx>,
    user_subscribed_channels: HashMap<String, HashSet<String>>
}

impl UserManager {

    pub fn new() -> Self  {
        Self {
            users: HashMap::new(),
            user_subscribed_channels: HashMap::new(),
        }
    }

    pub fn register_user(&mut self, user_id:String, tx:Tx){
        self.users.insert(user_id.clone(), tx);
        self.user_subscribed_channels.insert(user_id, HashSet::new());
    }

    pub async fn subscribe(
        &mut self, 
        user_id:String, 
        channel:String 
    ){

        let try_channels = self.user_subscribed_channels.get_mut(&user_id);

        match try_channels {
            Some(channels) => {
                channels.insert(channel.clone());
            },
            None => {
                println!("{} do not have channels set", user_id);
                let mut subscribed_channels = HashSet::new();
                subscribed_channels.insert(channel.clone());
                self.user_subscribed_channels.insert(user_id.clone(), subscribed_channels);
            }
        }
    }

    pub async fn unsubscribe(
        &mut self, 
        user_id:String, 
        channel:String 
    ){

        let try_channels = self.user_subscribed_channels.get_mut(&user_id);

        match try_channels {
            Some(channels) => {
                channels.remove(&channel);
            },
            None => {
                println!("{} do not have any subscribed channels", user_id);
            }
        }

    }

    pub async fn remove_user(
        &mut self,
        user_id:String,
    ) -> Option<HashSet<String>>{
        let channels = self.user_subscribed_channels.remove(&user_id);
        self.users.remove(&user_id);

        channels
    }

    // sends message on tx side for each user
    pub async fn emit_messages(&self, message:String, users:&HashSet<String>){
        println!("emitting message : {}, for users : {:?}", message, users);

        for user in users {
            let try_tx = self.users.get(user);
            match try_tx{
                Some(tx) => {
                    let res = tx.send(message.clone());

                    if let Err(e) = res {
                        println!("Error : {}, while sending message from tx for user : {}", e, user);
                    }

                },
                None => {
                    println!("no tx found for user : {}", user);
                }
            }

        }
    }       

    


}