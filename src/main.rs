#![feature(result_option_inspect)]
#![feature(async_closure)]
#![feature(once_cell)]
#![feature(unwrap_infallible)]
#![feature(hash_drain_filter)]
#![feature(linked_list_cursors)]

use std::{io, env};
use std::sync::{Arc};
use client::parser_v2::account_manager::account_pool_builder::AccountPoolBuilder;
use client::parser_v2::parser::ParserBuilder;
use client::parser_v2::task_publisher::{TaskPublisherMod, TaskPublisherBuilder};
use client::settings::{get_settings};
use log::LevelFilter;
use std::io::Write;

mod utils;
mod client;
mod reddit;
mod commons;


#[tokio::main(flavor = "multi_thread", worker_threads = 50)]
async fn main() -> Result<(), io::Error>{
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} {} [{}] - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter_level(LevelFilter::Debug)
        .init();
    let settings = get_settings();
    let mut parser = ParserBuilder::new(
        TaskPublisherBuilder::new(TaskPublisherMod::Manual, settings.clone(), 1000),
        AccountPoolBuilder::new(settings.clone())
    ).build().await;
    parser.start().await;
    Ok(())
} 
