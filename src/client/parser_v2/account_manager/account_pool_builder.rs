use std::{sync::Arc, collections::{HashMap, LinkedList}};

use log::info;
use tokio::sync::Mutex;
use futures::StreamExt;

use crate::{client::{settings::SettingsPtr}, commons::social_network::{dispatch_social_network, SocialNetworkEnum}};

use super::{account::{AccountPoolPtr, AccountPtr}, account_pool::AccountPool};

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
        for social_netowork in self.settings.social_network_settings.keys() {
            let accounts: LinkedList<AccountPtr> = dispatch_social_network(
                self.settings.clone(), 
                *social_netowork, 
                |settings, social_netowork_ptr| 
                    social_netowork_ptr.prepare_accounts(settings.clone())
            ).unwrap()
            .into_iter()
            .map(|account| Arc::new(account))
            .collect();

            info!("start account authorization");

            tokio_stream::iter(accounts.iter()).for_each_concurrent(8, |account| async {
                account.auth().await;
            }).await;

            info!("all accounts successfully authorized");

            self.social_network_accounts_map.insert(
                *social_netowork,
                Mutex::new(accounts)
            );
        }
    }

    pub async fn build(mut self) -> AccountPoolPtr {
        self.read_accounts_from_settings().await;
        return AccountPool::new(self.social_network_accounts_map).await;
    }
}
