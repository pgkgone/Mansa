use std::{collections::{HashMap, LinkedList}, sync::Arc, time::Duration};

use derivative::Derivative;
use tokio::sync::{Mutex, MutexGuard};

use crate::{commons::social_network::SocialNetworkEnum, utils::time::get_timestamp};

use super::account::{AccountPtr};

type AccountQueue = LinkedList<AccountPtr>;
type SocialNetworkAccountsMap = HashMap<SocialNetworkEnum, Mutex<AccountQueue>>;

type AccountPoolPtr = Arc<AccountPool>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct AccountPool {
    #[derivative(Debug="ignore")]
    pub (self) accounts_queue: SocialNetworkAccountsMap
}

impl AccountPool {

    pub async fn new(accounts: HashMap<SocialNetworkEnum, Mutex<LinkedList<AccountPtr>>>) -> AccountPoolPtr {
        let account_pool_ptr = Arc::new(AccountPool { 
            accounts_queue: accounts
        });
        return account_pool_ptr;
    }

    pub async fn get_account(&self, social_network: SocialNetworkEnum) -> AccountPtr {
        let social_net_accounts = self.accounts_queue.get(&social_network).expect("can't find netowork in accountpool");

        let mut guard_accounts = social_net_accounts.lock().await;
        let account = guard_accounts.front_mut().expect("Account queue should not be empty");

        let mut wg_session = account.session.write().await;
        let session = wg_session.as_mut().expect("Account should be authorized");

        let account_refresh_timestamp = session.retrieve_timestamp + session.millis_to_refresh;
        let account_clone = account.clone();
        if session.requests_limit > 0 {
            session.requests_limit -= 1;
            if session.requests_limit == 0 {
                std::mem::drop(wg_session);
                self.move_end(
                    guard_accounts.pop_front().expect("Account queue should not be empty"), 
                    account_refresh_timestamp, 
                    guard_accounts
                ).await;
            }
        } else {
            std::mem::drop(wg_session);
            std::mem::drop(guard_accounts);
            tokio::time::sleep(Duration::from_millis(account_refresh_timestamp - get_timestamp())).await;
        }
        return account_clone;
    }

    pub async fn move_end<'a>(
        &self, 
        account: AccountPtr, 
        account_refresh_timestamp: u64, 
        mut queue_guard: MutexGuard<'a, AccountQueue>
    ) -> MutexGuard<'a, AccountQueue> {
        let mut cursor = queue_guard.cursor_back_mut();
        while cursor.current().is_some() {
            let mut queue_account_referesh: u64 = 0;
            {
                let queue_account = cursor.current().unwrap();
                let opt_queue_account_guard = queue_account.session.read().await;
                let queue_account_guard = opt_queue_account_guard.as_ref().expect("access unauthorized session");
                queue_account_referesh = queue_account_guard.retrieve_timestamp + queue_account_guard.millis_to_refresh;
            }
            if queue_account_referesh < account_refresh_timestamp {
                cursor.insert_after(account);
                return queue_guard;
            } else {
                cursor.move_prev();
            }
        }
        queue_guard.push_front(account);
        return queue_guard;
    }
}