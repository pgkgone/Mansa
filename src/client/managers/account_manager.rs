use std::{collections::HashMap, sync::{Arc}, fs, path::Path};
use log::{info, error};
use priority_queue::PriorityQueue;
use reqwest::{ClientBuilder};

use crate::{generic::social_network::{SocialNetworkEnum, dispatch_social_network_async}, utils::{file_reader::read_json_from_file, time::get_timestamp}, client::{settings::{Account, Proxy, SettingsPtr}, http_client::HttpAuthData}};

pub type AccountPtr = Arc<Account>;
pub type ReqwestClientPtr = Arc<reqwest::Client>;
type StateSave = (HashMap<SocialNetworkEnum, Vec<AccountPtr>>, HashMap<AccountPtr, Proxy>);

pub enum DistributionStrategy {
    Uniform, 
    Unique,
    NoProxy
}

#[derive(Debug)]
pub struct AccountManager {
    account_manager: AccountManagerBuilder
}

impl AccountManager {
    
    pub fn get_account(&mut self, social_network: &SocialNetworkEnum) -> Option<(AccountPtr, HttpAuthData)> {

        let current_timestamp = get_timestamp();

        let social_network_accounts = self.account_manager
            .account_pool
            .pool.get_mut(&social_network).expect("no such social network");

        let account = social_network_accounts.pop();

        if account.is_some() {
            let account_uw = account.unwrap();
            if account_uw.1.requests_limit <= 0 {
                if account_uw.1.retrieve_timestamp + account_uw.1.millis_to_refresh < current_timestamp {
                    return Some((account_uw.0, account_uw.1));
                } else {
                    social_network_accounts.push(account_uw.0, account_uw.1);
                    return None;
                }
            } else {
                return Some((account_uw.0, account_uw.1));
            }
        } else {
            return None;
        }
    }

    pub fn get_accounts(&mut self, social_networks: &Vec<SocialNetworkEnum>) -> HashMap<SocialNetworkEnum, (AccountPtr, HttpAuthData)> {
        let mut hash_map: HashMap<SocialNetworkEnum, (AccountPtr, HttpAuthData)> = HashMap::new();
        for social_network in social_networks.iter() {
            match self.get_account(social_network).as_ref() {
                Some(account) => {hash_map.insert(*social_network, (account.0.clone(), account.1.clone()));},
                _ => ()
            }
        }
        return hash_map;
    }

    pub fn add_account(&mut self, account_ptr: AccountPtr, http_auth_data: HttpAuthData) {
        let mut pq = self
            .account_manager
            .account_pool
            .pool
            .get_mut(&account_ptr.social_network)
            .unwrap()
            .push(
                account_ptr, 
                http_auth_data
            );
    }

    pub fn get_client(&self, account_ptr: AccountPtr) -> Option<&ReqwestClientPtr> {

        return self.account_manager.account_client_map.get(&account_ptr);

    }

    pub fn update_auth_data(&mut self, account_ptr: AccountPtr, auth_data: &HttpAuthData) {

        self.account_manager
            .account_auth_data_map
            .insert(account_ptr.clone(), auth_data.clone());
        
        self.account_manager
            .account_pool
            .pool
            .get_mut(&account_ptr.social_network).expect("no such social net")
            .change_priority(&account_ptr, auth_data.clone());

    }

    pub fn get_auth_data(&mut self, account_ptr: AccountPtr) -> Option<&HttpAuthData> {
        return self.account_manager.account_auth_data_map.get(&account_ptr);
    }

    fn set_account_pool(&mut self) {
        for (k, v) in self.account_manager.social_network_accounts_map.iter() {
            let mut p_q: PriorityQueue<AccountPtr, HttpAuthData> = PriorityQueue::new();
            for account in v {
                p_q.push(
                    account.clone(), 
                    self.account_manager
                        .account_auth_data_map
                        .get(account)
                        .unwrap()
                        .clone()
                );
            }
            self.account_manager.account_pool.pool.insert(*k, p_q);
        }
    }

}

#[derive(Debug)]
pub struct AccountManagerBuilder {
    pub settings: Option<SettingsPtr>,
    pub social_network_accounts_map: HashMap<SocialNetworkEnum, Vec<AccountPtr>>,
    pub account_proxy_map: HashMap<AccountPtr, Proxy>,
    pub account_client_map: HashMap<AccountPtr, ReqwestClientPtr>,
    pub account_auth_data_map: HashMap<AccountPtr, HttpAuthData>,
    pub account_pool: AccountPool,
}

#[derive(Debug)]
pub struct AccountPool {
    pub pool: HashMap<SocialNetworkEnum, PriorityQueue<AccountPtr, HttpAuthData>>
}

impl AccountPool {
    pub fn new() -> AccountPool {
        return AccountPool { pool: HashMap::new() }
    }
}

impl AccountManagerBuilder {

    pub fn new(strategy: DistributionStrategy, settings: SettingsPtr) -> AccountManagerBuilder {
        let mut am = AccountManagerBuilder {
            social_network_accounts_map: HashMap::new(),
            account_client_map: HashMap::new(),
            account_auth_data_map: HashMap::new(),
            account_proxy_map: HashMap::new(),
            account_pool: AccountPool::new(),
            settings: Some(settings)
        };
        am.set_social_net_account_map();
        am.set_account_proxy_map(strategy);
        am.set_account_client_map();
        return am;
    }

    pub fn load_from_state() -> AccountManagerBuilder {
        let loaded_state: StateSave = read_json_from_file(Path::new("save.state"));
        let mut am = AccountManagerBuilder { 
            social_network_accounts_map: loaded_state.0,
            account_client_map: HashMap::new(),
            account_auth_data_map: HashMap::new(),
            account_proxy_map: loaded_state.1,
            account_pool: AccountPool::new(),
            settings: None
        };
        am.set_account_client_map();
        return am;
    }

    pub fn save_state(&self) {
        let json = serde_json::to_string(&(&self.social_network_accounts_map, &self.account_proxy_map)).unwrap();
        match fs::write("save.state", json.as_bytes()) {
            Ok(_) => {},
            Err(err) => {
                println!("{}", err);
                panic!("unable to save state") 
            },
        };
    }

    pub async fn auth<'a: 'b, 'b>(mut account_manager_builder: AccountManagerBuilder) -> AccountManager
    {
        info!("start AccountManager auth process");
        for (k, v) in account_manager_builder.social_network_accounts_map.iter() {

            info!("process auth for {}", k.to_string());
            for account in v.iter() {

                let r = dispatch_social_network_async(
                    (account.clone(), account_manager_builder.account_client_map.get(account).unwrap().to_owned()),
                    account.social_network,
                    async move |
                    data,
                    network| {
                        return network.auth(data.0, data.1).await;
                    }
                ).await
                .inspect_err(
                    |e| error!("{} for account: login: {}  password: {} social_network:{}",
                        e, 
                        account.login.as_ref().unwrap_or(&"no login for that social net".to_string()), 
                        account.password.as_ref().unwrap_or(&"no password for that social net".to_string()), k.to_string()
                    )
                ).unwrap();
                
                let r_token = r.token.clone();
                account_manager_builder.account_auth_data_map.insert(account.clone(), r);
                println!("{}", r_token);
            }
        }
        let mut aam = AccountManager {
            account_manager: account_manager_builder
        };
        aam.set_account_pool(); 
        return aam;
    }

    fn set_social_net_account_map(&mut self) {
        for social_network in self.settings.as_ref().expect("settings file for account manager not set").social_network_settings.iter() {
            let mut arc_accounts: Vec<AccountPtr> = Vec::with_capacity(social_network.accounts.len()); 
            for account in social_network.accounts.iter() {
                arc_accounts.push(Arc::new(account.clone()))
            }
            self.social_network_accounts_map.insert(social_network.social_network, arc_accounts);
        }
    }

    fn set_account_proxy_map(&mut self, strategy: DistributionStrategy) {
        let strategy = if self.settings.as_ref().unwrap().general_settings.disable_proxy { DistributionStrategy::NoProxy } else { strategy };
        match strategy {
            DistributionStrategy::Uniform => self.uniform_distribute(),
            DistributionStrategy::Unique => self.unique(),
            DistributionStrategy::NoProxy => self.no_proxy(),
        };
    }

    fn set_account_client_map(&mut self) {
        for (k, v) in self.social_network_accounts_map.iter() {
            for account in v.iter() {
                let account_proxy = self.account_proxy_map.get(account).cloned();
                let reqwest_client = AccountManagerBuilder::create_reqwest_client(
                    account.clone(), 
                    account_proxy
                );
                self.account_client_map.insert(
                    account.clone(), 
                    reqwest_client.clone()
                );
            }
        }
    }

    fn uniform_distribute(&mut self) {
        for (social_network, accounts) in self.social_network_accounts_map.iter() {
            let proxy_count = self.settings.as_ref().unwrap().general_settings.proxies.len();
            if proxy_count == 0 {
                panic!("Use uniform distribution on empty proxy list! Use another distribution strategy!");
            }
            let proxies = &self.settings.as_ref().unwrap().general_settings.proxies;
            for (ndex, account) in accounts.iter().enumerate() {
                self.account_proxy_map.insert(
                    account.clone(), 
                    proxies[ndex % proxy_count].clone()
                );
            }
        }
    }

    fn unique(&mut self) {
        for (social_network, accounts) in self.social_network_accounts_map.iter() {
            let mut collection_len = usize::max(accounts.len(), self.settings.as_ref().unwrap().general_settings.proxies.len());
            if collection_len == 0 {
                panic!("Number of accounts or proxies empty! Add accounts or change distribution strategy");
            } 
            let proxies = &self.settings.as_ref().unwrap().general_settings.proxies;
            for ndex in 0..collection_len {
                self.account_proxy_map.insert(
                    accounts[ndex].clone(), 
                    proxies[ndex].clone()
                );
            }
        }
    }

    fn no_proxy(&self) {}

    fn create_reqwest_client(account_ptr: AccountPtr, proxy_ptr: Option<Proxy>) -> ReqwestClientPtr {
        let mut client_builder: Option<ClientBuilder> = None;
        if proxy_ptr.is_some() {
            client_builder = Some(reqwest::Client::builder()
                .user_agent("PostmanRuntime/7.29.0")
                .proxy(proxy_ptr.as_ref().unwrap().into())
            );
        } else {
            client_builder = Some(
                reqwest::Client::builder()
                .user_agent("PostmanRuntime/7.29.0")
            );
        }
        return Arc::new(client_builder.unwrap().build().unwrap());
    }
}