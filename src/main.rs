#![feature(result_option_inspect)]
#![feature(async_closure)]
#![feature(once_cell)]
#![feature(unwrap_infallible)]
#![feature(hash_drain_filter)]
#![feature(linked_list_cursors)]

use std::slice::Split;
use std::sync::Arc;
use std::time::Duration;
use std::{io, env};
use client::parser_v2::account_manager::account_pool_builder::AccountPoolBuilder;
use client::parser_v2::parser::ParserBuilder;
use client::parser_v2::statistics::{STATISTICS};
use client::parser_v2::task_publisher::{TaskPublisherMod, TaskPublisherBuilder};
use client::settings::{get_settings};
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::{Config};
use log4rs::config::{Appender, Root, Logger};

mod utils;
mod client;
mod reddit;
mod commons;

#[tokio::main(flavor = "multi_thread", worker_threads = 50)]
async fn main() -> Result<(), io::Error>{
    init_logger();
    run_statistics_printing();
    let settings = get_settings();
    let mut parser = ParserBuilder::new(
        TaskPublisherBuilder::new(TaskPublisherMod::Manual, settings.clone(), 1000),
        AccountPoolBuilder::new(settings.clone())
    ).build().await;
    parser.start().await;
    Ok(())
} 


pub fn init_logger() {
    let file_appender = FileAppender::builder()
        .build("log/logs.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(file_appender)))
        .logger(Logger::builder().build("mansa", LevelFilter::Debug))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
        .unwrap();

    let handle = log4rs::init_config(config).unwrap();
}

pub fn run_statistics_printing() {
    tokio::spawn(async move {
        loop {
            println!("{:?}", STATISTICS.get_snapshot());
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}