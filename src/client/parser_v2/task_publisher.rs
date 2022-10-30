
use std::{time::Duration, sync::Arc};

use futures::StreamExt;
use log::info;
use tokio::sync::mpsc::{self, Sender, Receiver};

use crate::{commons::{parsing_tasks::{ParsingTaskStatus, ParsingTask}, social_network::dispatch_social_network}, client::{db::tasks_db::{Limit, get_tasks_sorted_by_exec_time, update_task_with_status, insert_tasks}, settings::{SettingsPtr, self}}};

pub type TaskPublisherPtr = Arc<TaskPublisher>;

#[derive(Clone)]
pub enum TaskPublisherMod {
    Recovery,
    Manual
}

pub struct TaskPublisher {
    limit: u64,
    sender: Sender<ParsingTask>,
    startup_mod: TaskPublisherMod,
    settings: SettingsPtr
}

impl TaskPublisher {

    pub fn new(task_publisher_mod: TaskPublisherMod, settings: SettingsPtr, limit: u64) -> (TaskPublisherPtr, Receiver<ParsingTask>)  {
        let (sd, rc) = mpsc::channel(limit as usize);
        return (
            Arc::new(TaskPublisher { 
                limit: limit, 
                sender: sd,
                startup_mod: task_publisher_mod,
                settings: settings
            }),
            rc
        )
    }

    pub async fn start(&self) {
        match self.startup_mod {
            TaskPublisherMod::Recovery => self.push_tasks(
                self.fetch_recovery().await
            ).await,
            TaskPublisherMod::Manual => self.read_settings_tasks(self.settings.clone()).await
        }

        loop {
            self.push_tasks(
                self.fetch().await
            ).await;
            tokio::time::sleep(Duration::from_secs(1)).await;   
        }
    }

    async fn push_tasks(&self, tasks: Vec<ParsingTask>) {
        tokio_stream::iter(
            tasks
        )
        .for_each_concurrent(8, |item| async {
            update_task_with_status(item._id.unwrap(), ParsingTaskStatus::Processing).await;
            self.sender.send(item).await.expect("error while sending parsing task");
        })
        .await;
    }

    async fn fetch(&self) -> Vec<ParsingTask> {
        self.fetch_tasks(
            vec![
                ParsingTaskStatus::New
            ]
        ).await
    }

    async fn fetch_recovery(&self) -> Vec<ParsingTask> {
        self.fetch_tasks(
            vec![
                ParsingTaskStatus::Processing
            ]
        ).await
    }

    async fn fetch_tasks(&self, statuses: Vec<ParsingTaskStatus>) -> Vec<ParsingTask> {
        get_tasks_sorted_by_exec_time(
                statuses,
                Limit::Limit(self.limit)
        ).await
    }

    async fn read_settings_tasks(&self, settings: SettingsPtr) {
        let mut parsing_tasks: Vec<ParsingTask> = Vec::new();

        for social_network_settings in settings.social_network_settings.values() {
            
            let mut tasks = dispatch_social_network(
                settings.clone(), 
                social_network_settings.social_network, 
                |settings, social_network_ptr| 
                    social_network_ptr.prepare_parsing_tasks(settings) )
                .expect("unable to process tasks from settings file");
            info!("{:#?}", tasks);
            parsing_tasks.append(&mut tasks);

        }

        insert_tasks(&parsing_tasks).await;
    }

}

pub struct TaskPublisherBuilder {
    limit: u64,
    startup_mod: TaskPublisherMod,
    settings: SettingsPtr
}

impl TaskPublisherBuilder {

    pub fn new(startup_mode: TaskPublisherMod, settings: SettingsPtr, limit: u64) -> TaskPublisherBuilder {
        return TaskPublisherBuilder { 
            startup_mod: startup_mode, 
            settings: settings, 
            limit: limit 
        };
    }

    pub async fn build(self) -> (TaskPublisherPtr, Receiver<ParsingTask>) {
        return TaskPublisher::new(self.startup_mod, self.settings, self.limit);
    }

}