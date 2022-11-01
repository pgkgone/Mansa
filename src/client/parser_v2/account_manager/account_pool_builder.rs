use std::{
    sync::Arc, 
    collections::{
        HashMap, 
        LinkedList
    }
};

use log::info;
use tokio::sync::Mutex;
use futures::StreamExt;

use crate::{
    client::{
        settings::SettingsPtr
    }, 
    commons::social_network::{
        SocialNetworkEnum, 
        SOCIAL_NETWORKS
    }
};

use super::{
    account::{
        AccountPoolPtr, 
        AccountPtr
    }, 
    account_pool::AccountPool
};

pub struct AccountPoolBuilder {
    settings: SettingsPtr,
    social_network_accounts_map: HashMap<SocialNetworkEnum, Mutex<LinkedList<AccountPtr>>>
}

impl AccountPoolBuilder {

    pub fn new(settings: SettingsPtr) -> AccountPoolBuilder {
        return Self { 
            settings,
            social_network_accounts_map: HashMap::new()
         }
    }
 
    async fn read_accounts_from_settings(&mut self) {
        for social_network in self.settings.social_network_settings.keys() {

            let accounts: LinkedList<AccountPtr> = SOCIAL_NETWORKS.get(&social_network)
                .expect("No such social network!")
                .prepare_accounts(self.settings.clone()).unwrap()
                .into_iter()
                .map(|account| Arc::new(account))
                .collect();

            info!("start account authorization");

            tokio_stream::iter(accounts.iter()).for_each_concurrent(8, |account| async {
                account.auth().await;
            }).await;

            info!("all accounts successfully authorized");

            self.social_network_accounts_map.insert(
                *social_network,
                Mutex::new(accounts)
            );
        }
    }

    pub async fn build(mut self) -> AccountPoolPtr {
        self.read_accounts_from_settings().await;
        return AccountPool::new(self.social_network_accounts_map).await;
    }
}
