#![feature(result_option_inspect)]
#![feature(async_closure)]
#![feature(once_cell)]
#![feature(unwrap_infallible)]
#![feature(hash_drain_filter)]
#![feature(nll)]

use std::{path::Path, io::{self, Error}, env, cell::{RefCell, UnsafeCell}, rc::Rc, sync::{Arc, Mutex}, ops::DerefMut, time::Instant};
use core::ops::Deref;

use chrono::{DateTime, NaiveDateTime};
use client::{settings::{Settings, get_settings}, parser::Parser, managers::task_manager::ParsingTask, db::{entities_db::insert_entities, tasks_db::{insert_tasks, Limit, get_tasks_sorted_by_exec_time}}};
use env_logger::{Env, Builder, Target};
use futures::{Future, FutureExt};
use generic::social_network::SocialNetworkEnum;
use lazy_static::__Deref;
use log::{error, info};
use priority_queue::PriorityQueue;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use utils::time::get_timestamp;
use futures::{StreamExt, Stream};

use crate::{utils::file_reader, generic::social_network::{SOCIAL_NETWORKS, SocialNetwork}, client::{managers::{account_manager::{AccountManagerBuilder, DistributionStrategy}, task_manager::TaskManager}, db::tasks_db::{get_tasks_grouped_by_social_network, GroupedTasks}}};

mod utils;
mod client;
mod reddit;
mod generic;

#[tokio::main]
async fn main() -> Result<(), io::Error>{
    env_logger::init();
    info!("test");
    let settings = get_settings();
    let account_manager_builder = AccountManagerBuilder::new(DistributionStrategy::NoProxy, settings.clone());
    let account_manager = Arc::new(RwLock::new(AccountManagerBuilder::auth(account_manager_builder).await));
    let task_manager = Arc::new(RwLock::new(TaskManager::new(settings.clone()).await));
    let parser = Parser::new(account_manager, task_manager);
    parser.start().await; 
    return Ok(())
} 
