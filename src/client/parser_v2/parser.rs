use std::{sync::{atomic::AtomicUsize, Arc}};

use log::debug;
use tokio::sync::{mpsc::Receiver, Notify};

use crate::{
    commons::{
        parsing_tasks::{
            ParsingTask
        }, 
        social_network::{
            SOCIAL_NETWORKS
        }
    }
};

use super::{
    task_publisher::{
        TaskPublisherBuilder, TaskPublisherPtr
    }, 
    account_manager::{
        account_pool::{
            AccountPool
        }, 
        account::AccountPtr, 
        account_pool_builder::AccountPoolBuilder
    }, 
    statistics::STATISTICS
};

pub type AccountPoolPtr = Arc<AccountPool>;
pub type ParserThreadCounterPtr = Arc<ParserThreadCounter>;
pub struct ParserThreadCounter {
    number_concurrent_tasks: AtomicUsize,
    thread_available_notification: Notify,
    limit: usize
}

impl ParserThreadCounter {

    pub fn new(limit: usize) -> ParserThreadCounter {
        return ParserThreadCounter { 
            number_concurrent_tasks: AtomicUsize::new(0),
            thread_available_notification: Notify::new(),
            limit
        }
    }

    pub async fn increase(&self) -> usize {
        let current_number_of_threads = self.number_concurrent_tasks.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        STATISTICS.increase_current_running_threads();
        if current_number_of_threads > 20 {
            debug!("waiting for thread notification");
            self.wait_available_thread().await;
        }
        return self.number_concurrent_tasks.fetch_add(0, std::sync::atomic::Ordering::Relaxed);
    }

    pub async fn decrease(&self) -> usize {
        let current_number_of_threads = self.number_concurrent_tasks.fetch_sub(1, std::sync::atomic::Ordering::Relaxed) - 1;
        STATISTICS.decrease_current_running_threads();
        self.thread_available_notification.notify_one();
        return current_number_of_threads;
    }

    async fn wait_available_thread(&self) {
        self.thread_available_notification.notified().await;
    }


}

pub struct Parser {
    task_publisher: TaskPublisherPtr,
    task_receiver: Receiver<ParsingTask>,
    account_pool: AccountPoolPtr,
    thread_counter: ParserThreadCounterPtr
}

impl Parser {

    pub async fn start(&mut self) {

        self.run_task_publisher();

        loop {

            debug!("starts parsing loop");

            let current_number_of_threads = self.thread_counter.increase().await;

            debug!("current number of parsing threads: {}", current_number_of_threads);

            debug!("receiving task");

            let parsing_task = self.task_receiver
                .recv()
                .await;
            
            let parsing_task = parsing_task.expect("Task channel closed unexpectedly");
            
            debug!("task successfully received");
            debug!("receiving account");

            let account = self.account_pool
                .get_account(parsing_task.social_network).await; 

            debug!("account successfully received");

            let thread_counter = self.thread_counter.clone();

            debug!("spawning async parsing task");

            tokio::spawn(async move {
                STATISTICS.increase_started_parsing_tasks();
                Self::parse(parsing_task, account).await;
                thread_counter.decrease().await;
                STATISTICS.increase_successful_parsing_tasks();
            });
        }
    }

    async fn parse(task: ParsingTask, account: AccountPtr) {
        let social_network = task.social_network.clone();
        SOCIAL_NETWORKS.get(&social_network)
            .expect("No such social network!")
            .parse(task, account)
            .await;
    }

    pub fn run_task_publisher(&self) {
        let task_publisher_ptr_clone = self.task_publisher.clone();

        tokio::spawn( async move {
            task_publisher_ptr_clone.start().await;
        });
    }
}

pub struct ParserBuilder {
    task_publisher_builder: TaskPublisherBuilder,
    account_pool_builder: AccountPoolBuilder
}

impl ParserBuilder {

    pub fn new(task_publisher_builder: TaskPublisherBuilder, account_pool_builder: AccountPoolBuilder) -> ParserBuilder {
        return ParserBuilder { 
            task_publisher_builder: task_publisher_builder, 
            account_pool_builder: account_pool_builder
        }
    }

    pub async fn build(self) -> Parser {

        let (task_publisher, receiver) = self.task_publisher_builder.build().await;
        
        return Parser {
            task_publisher: task_publisher,
            task_receiver: receiver,
            account_pool: self.account_pool_builder.build().await,
            thread_counter: Arc::new(ParserThreadCounter::new(20))
        }
    }



}