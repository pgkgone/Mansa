#![feature(result_option_inspect)]
#![feature(async_closure)]
#![feature(once_cell)]
#![feature(unwrap_infallible)]
#![feature(hash_drain_filter)]

use std::{io::{self}, sync::{Arc}};
use client::{settings::{get_settings}, parser::Parser};
use tokio::sync::RwLock;

use crate::{client::{managers::{account_manager::{AccountManagerBuilder, DistributionStrategy}, task_manager::TaskManager}}};

mod utils;
mod client;
mod reddit;
mod generic;

#[tokio::main]
async fn main() -> Result<(), io::Error>{
    env_logger::init();
    let settings = get_settings();
    let account_manager_builder = AccountManagerBuilder::new(DistributionStrategy::NoProxy, settings.clone());
    let account_manager = Arc::new(RwLock::new(AccountManagerBuilder::auth(account_manager_builder).await));
    let task_manager = Arc::new(RwLock::new(TaskManager::new(settings.clone()).await));
    let parser = Parser::new(account_manager, task_manager);
    parser.start().await; 
    return Ok(())
} 
