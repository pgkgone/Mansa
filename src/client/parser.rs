use std::{error::Error, sync::{Arc}, collections::HashMap, thread, time::Duration};

use log::{info, debug};

use crate::{generic::social_network::{dispatch_social_network_async, SocialNetworkEnum}, client::managers::task_manager::ParsingTask};

use super::{settings::{SettingsPtr, Account}, http_client::HttpAuthData, managers::{account_manager::{AccountManager, AccountPtr}, task_manager::TaskManager}};

pub type AccountManagerPtr = Arc<tokio::sync::RwLock<AccountManager>>;
pub type TaskManagerPtr = Arc<tokio::sync::RwLock<TaskManager>>;

pub struct Parser {
    pub account_manager: AccountManagerPtr,
    pub task_manager: TaskManagerPtr
}

impl Parser {

    pub fn new(account_manager: AccountManagerPtr, task_manager: TaskManagerPtr) -> Parser {
        return Parser{
            account_manager,
            task_manager
        }
    }

    pub async fn start(&self) {
        info!("start parsing loop");
        while true {
            self.parse().await;
        }
    }

    async fn parse(&self) {

        let mut task_manager_ptr = self.task_manager.clone();
        let mut account_manager_ptr = self.account_manager.clone();

        debug!("locking task manager 1");
        let mut task_manager_locked = self.task_manager.write().await;
        let mut tasks = task_manager_locked.get_grouped_tasks();
        drop(task_manager_locked);

        let social_nets_with_tasks: Vec<SocialNetworkEnum> = tasks.keys().cloned().collect();

        debug!("locking account manager 1");
        let mut account_manager_locked = account_manager_ptr.write().await;
        let mut accounts = account_manager_locked.get_accounts(&social_nets_with_tasks);
        drop(account_manager_locked);

        let unused_tasks = Self::get_unused_tasks(&mut accounts, &mut tasks);

        debug!("locking task manager 2");
        let mut task_manager_locked = self.task_manager.write().await;
        task_manager_locked.add_parsing_tasks(unused_tasks);
        drop(task_manager_locked);

        for accounts in accounts.iter() {
            tokio::spawn(
                Self::parse_tasks(
                    self.account_manager.clone(), 
                    self.task_manager.clone(), 
                    accounts.1.clone(), 
                    tasks.get(accounts.0).unwrap().clone())
                );
        }

        thread::sleep(Duration::from_millis(1000));

    }


    async fn parse_tasks(account_manager_ptr: AccountManagerPtr, task_manager_ptr: TaskManagerPtr, account: (AccountPtr, HttpAuthData), tasks_to_parse: Vec<ParsingTask>) {
        info!("start parsing task");
        let result: (Option<HttpAuthData>, Vec<ParsingTask>) = dispatch_social_network_async(
            (account_manager_ptr.clone(), account.clone(), tasks_to_parse),
            account.0.social_network,
            async move |data, network_ptr| {
                return network_ptr.parse(data.0, data.1, data.2).await;
            })
            .await;

        if result.0.is_some() {
            //locking
            let http_uw = result.0.unwrap();
            let mut account_manager_locked = account_manager_ptr.write().await;
            //do we really need this?
            account_manager_locked.update_auth_data(account.0.clone(), &http_uw);
            account_manager_locked.add_account(account.0.clone(), http_uw);
        }

        if !result.1.is_empty() {
            //locking
            let mut task_manager_locked = task_manager_ptr.write().await;
            task_manager_locked.add_parsing_tasks(result.1);
        }
    

    }


    fn get_unused_tasks(accounts: &mut HashMap<SocialNetworkEnum, (Arc<Account>, HttpAuthData)>, tasks: &mut HashMap<SocialNetworkEnum, Vec<ParsingTask>>) -> Vec<ParsingTask> {

        let mut unused_tasks: Vec<ParsingTask> = Vec::with_capacity(10);
        tasks.drain_filter(|social_net, parsing_tasks| {
            return match accounts.contains_key(social_net) {
                true => false,
                false => {
                    unused_tasks.append(parsing_tasks);
                    return true;
                },
            }
        });
        unused_tasks
            .iter_mut()
            .for_each(|item| {
                item.execution_time = item.execution_time + 2000;
            });

        return unused_tasks;
    }

}