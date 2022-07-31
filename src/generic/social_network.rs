use std::{collections::HashMap, future::Future, error::Error};

use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{error, info};
use serde::{Serialize, Deserialize};
use strum::{EnumIter, Display};
use crate::{client::{http_client::HttpAuthData, settings::ParsingTaskSettings, parser::AccountManagerPtr, managers::{account_manager::{AccountPtr, ReqwestClientPtr}, task_manager::ParsingTask}}, reddit::reddit::Reddit};



pub type SocialNetworkPtr = Box<dyn SocialNetwork + Sync>;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq, EnumIter, Display)]
pub enum SocialNetworkEnum {
    Reddit
}


lazy_static! {
    pub static ref SOCIAL_NETWORKS: HashMap<String, SocialNetworkPtr> = {
        let mut map: HashMap<String, SocialNetworkPtr> = HashMap::new();
        map.insert(SocialNetworkEnum::Reddit.to_string(), Box::new(Reddit::default()));
        return map;
    };
}

#[async_trait]
pub trait SocialNetwork {
    async fn auth(&self, account_ptr: AccountPtr, client_ptr: ReqwestClientPtr) -> Result<HttpAuthData, Box<dyn Error + Send + Sync>>;
    async fn parse(&self, account_manager_ptr: AccountManagerPtr, account: (AccountPtr, HttpAuthData), parsing_task: Vec<ParsingTask>) -> (Option<HttpAuthData>, Vec<ParsingTask>);
    fn process_settings_tasks(&self, tasks: &Vec<ParsingTaskSettings>) -> Result<Vec<ParsingTask>, Box<dyn Error>>; 
}

pub fn dispatch_social_network<T, R, F>(
    data: T, 
    social_network: SocialNetworkEnum, 
    action: F
) -> Result<R, Box<dyn Error>>
where
    F: Fn(T, &'static SocialNetworkPtr) -> Result<R, Box<dyn Error>>
{
    match SOCIAL_NETWORKS.get(&social_network.to_string()) {
        Some(social_network_ptr) => {
            info!("socialnetwork successfully matched");
            return action(data, social_network_ptr);
        },
        None => {
            error!("unsuccessfully SOCIAL_NETWORK dispatch: {}", social_network.to_string());
            return Err(format!("unsuccessfully SOCIAL_NETWORK dispatch: {}", social_network.to_string()).into());
        }
    }

}

pub async fn dispatch_social_network_async<T, R, F, Fut>(
    data: T, 
    social_network: SocialNetworkEnum, 
    action: F
) -> R
where
    F: Fn(T, &'static SocialNetworkPtr) -> Fut,
    Fut: Future<Output = R>,
{
    match SOCIAL_NETWORKS.get(&social_network.to_string()) {
        Some(social_network_ptr) => {
            info!("socialnetwork successfully matched");
            return action(data, social_network_ptr).await;
        },
        None => {
            panic!("unsuccessfully SOCIAL_NETWORK dispatch: {}", social_network.to_string());
        }
    }

}