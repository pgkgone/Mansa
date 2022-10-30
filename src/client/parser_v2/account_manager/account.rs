use std::{sync::Arc};

use derivative::Derivative;
use log::{info, error};
use tokio::sync::{RwLock};

use crate::{client::{settings::{self, Proxy}}, commons::social_network::dispatch_social_network_async};

use super::account_pool::AccountPool;

pub type ProxyPtr = Box<Proxy>;
pub type ReqwestClientPtr = Arc<reqwest::Client>;
pub type AccountDataPtr = Arc<settings::Account>;
pub type AccountSessionPtr = Arc<RwLock<Option<AccountSession>>>;
pub type AccountPoolPtr = Arc<AccountPool>;
pub type AccountPtr = Arc<Account>;

#[derive(Derivative)]
#[derivative(Hash, Eq)]
pub struct Account {
    pub account_data: AccountDataPtr,
    #[derivative(Hash="ignore")]
    pub proxy: Option<ProxyPtr>,
    #[derivative(Hash="ignore")]
    pub session: AccountSessionPtr,
    #[derivative(Hash="ignore")]
    pub reqwest_client: Option<ReqwestClientPtr>
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        return self.account_data == self.account_data;
    }
}

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Hash, Eq)]
pub struct AccountSession {
    pub token: String,
    pub retrieve_timestamp: u64,
    pub millis_to_refresh: u64,
    pub requests_limit: usize,
}

impl Account {

    pub fn new(account_data: AccountDataPtr, proxy: Option<ProxyPtr>) -> Account {
        let mut account = Account { 
            account_data: account_data,
            proxy: proxy, 
            session: Arc::new(RwLock::new(None)), 
            reqwest_client: None
        };
        account.setup_reqwest_client();
        return account;
    }
    
    pub async fn auth(&self) 
    {
        info!("authenticate account {:?}", self.account_data);
        let session = dispatch_social_network_async(
            (self.account_data.clone(), self.reqwest_client.clone()), 
            self.account_data.social_network, 
            |(account_data, client_ptr), network| {
                network.auth(account_data, client_ptr.expect("Reqwest client should be set before authentication of account"))
            }
        ).await
        .inspect_err(
            |e| error!("{} for account: login: {}  password: {} social_network:{}",
                e, 
                self.account_data.login.as_ref().unwrap_or(&"no login for that social net".to_string()), 
                self.account_data.password.as_ref().unwrap_or(&"no password for that social net".to_string()), 
                self.account_data.social_network.to_string()
            )
        ).unwrap();
        let mut session_guard = self.session.write().await;
        session_guard.replace(session);
    }

    pub fn setup_reqwest_client(&mut self) {
        let mut client_builder = reqwest::Client::builder()
            .user_agent("PostmanRuntime/7.29.0");
        
        if self.proxy.is_some() {
            client_builder = client_builder.proxy(self.proxy.as_deref().unwrap().into());
        }

        self.reqwest_client = Some(
            Arc::new(client_builder.build().unwrap())
        );
    }

}