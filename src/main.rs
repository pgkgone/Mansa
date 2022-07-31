#![feature(result_option_inspect)]
#![feature(async_closure)]
#![feature(once_cell)]
#![feature(unwrap_infallible)]
#![feature(hash_drain_filter)]

use std::{path::Path, io, env, cell::{RefCell, UnsafeCell}, rc::Rc, sync::Arc, ops::DerefMut};
use core::ops::Deref;

use chrono::DateTime;
use client::{settings::{Settings, get_settings}, parser::Parser};
use env_logger::{Env, Builder, Target};
use lazy_static::__Deref;
use log::{error, info};
use priority_queue::PriorityQueue;
use tokio::sync::RwLock;

use crate::{utils::file_reader, generic::social_network::{SOCIAL_NETWORKS, SocialNetwork}, client::managers::{account_manager::{AccountManagerBuilder, DistributionStrategy}, task_manager::TaskManager}};

mod utils;
mod client;
mod reddit;
mod generic;


#[tokio::main]
async fn main() -> Result<(), io::Error>{
    /*
    env_logger::init();
    info!("test");
    let settings = get_settings();
    let account_manager_builder = AccountManagerBuilder::new(DistributionStrategy::NoProxy, settings.clone());
    let account_manager = Arc::new(RwLock::new(AccountManagerBuilder::auth(account_manager_builder).await));
    let task_manager = Arc::new(RwLock::new(TaskManager::new(settings.clone())));
    let parser = Parser::new(account_manager, task_manager);
    parser.start().await;
     */

    //let date = DateTime::parse_from_rfc2822("Sun, 31 Jul 2022 00:01:30 GMT").unwrap().timestamp();
    println!("{}", ((1658448546.0 as f64) as i64));
    return Ok(())
}
