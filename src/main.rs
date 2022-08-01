#![feature(result_option_inspect)]
#![feature(async_closure)]
#![feature(once_cell)]
#![feature(unwrap_infallible)]
#![feature(hash_drain_filter)]

use std::{path::Path, io, env, cell::{RefCell, UnsafeCell}, rc::Rc, sync::{Arc, Mutex}, ops::DerefMut, time::Instant};
use core::ops::Deref;

use chrono::{DateTime, NaiveDateTime};
use client::{settings::{Settings, get_settings}, parser::Parser, managers::task_manager::ParsingTask};
use env_logger::{Env, Builder, Target};
use generic::social_network::SocialNetworkEnum;
use lazy_static::__Deref;
use log::{error, info};
use priority_queue::PriorityQueue;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{utils::file_reader, generic::social_network::{SOCIAL_NETWORKS, SocialNetwork}, client::managers::{account_manager::{AccountManagerBuilder, DistributionStrategy}, task_manager::TaskManager}};

mod utils;
mod client;
mod reddit;
mod generic;


#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ParsingTask2 {
    pub _id: u64,
    pub execution_time: u64,
    pub url: u64,
    pub action_type: u64,
    pub social_network: SocialNetworkEnum
}

#[tokio::main]
async fn main() -> Result<(), io::Error>{
    env_logger::init();
    info!("test");
    let settings = get_settings();
    let account_manager_builder = AccountManagerBuilder::new(DistributionStrategy::NoProxy, settings.clone());
    let account_manager = Arc::new(RwLock::new(AccountManagerBuilder::auth(account_manager_builder).await));
    let task_manager = Arc::new(RwLock::new(TaskManager::new(settings.clone())));
    let parser = Parser::new(account_manager, task_manager);
    parser.start().await; 
    return Ok(())
}
