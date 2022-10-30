use std::{sync::{atomic::AtomicUsize, Arc}};

use log::debug;
use tokio::sync::{mpsc::Receiver, Notify};

use crate::{commons::{parsing_tasks::{ParsingTask, self}, social_network::dispatch_social_network_async}};

use super::{task_publisher::{TaskPublisher, TaskPublisherMod, TaskPublisherBuilder, self, TaskPublisherPtr}, account_manager::{account_pool::{AccountPool, self}, account::AccountPtr, account_pool_builder::AccountPoolBuilder}};

pub type AccountPoolPtr = Arc<AccountPool>;

pub struct Parser {
    task_publisher: TaskPublisherPtr,
    task_receiver: Receiver<ParsingTask>,
    account_pool: AccountPoolPtr,
    number_concurrent_tasks: AtomicUsize, 
    thread_available_notification: Arc<Notify>
}

impl Parser {

    pub async fn start(&mut self) {

        self.run_task_publisher();

        loop {

            debug!("starts parsing loop");

            let current_number_of_threads = self.add_thread().await;

            if current_number_of_threads == 1000 {
                self.wait_available_thread().await;
                self.sub_thread();
            }

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

            let notifier = self.thread_available_notification.clone();

            debug!("spawning async parsing task");

            tokio::spawn(async move {
                Self::parse(parsing_task, account).await;
                notifier.notify_one();
            });
        }
    }

    async fn parse(task: ParsingTask, account: AccountPtr) {
        let social_network = task.social_network.clone();
        dispatch_social_network_async(
            (task, account), 
            social_network, 
            |(task, account), social_net_ptr| {
                social_net_ptr.parse(task, account)
            }
        ).await;
    }

    async fn add_thread(&mut self) -> usize {
        return self.number_concurrent_tasks.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    }

    async fn sub_thread(&mut self) -> usize {
        return self.number_concurrent_tasks.fetch_sub(1, std::sync::atomic::Ordering::Relaxed) - 1;
    }

    async fn wait_available_thread(&mut self) {
        self.thread_available_notification.notified().await;
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
            number_concurrent_tasks: AtomicUsize::new(0),
            thread_available_notification: Arc::new(Notify::new()),
        }
    }



}