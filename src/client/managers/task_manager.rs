use std::{time::{SystemTime, UNIX_EPOCH}, collections::HashMap, hash::Hash, cmp::Reverse};

use log::{error, info};
use mongodb::bson::oid::ObjectId;
use priority_queue::PriorityQueue;
use serde::{Serialize, Deserialize};

use crate::{generic::social_network::{SocialNetworkEnum, dispatch_social_network}, utils::time::get_timestamp, client::settings::SettingsPtr};

#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ParsingTask {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub execution_time: u64,
    pub url: String,
    pub action_type: String,
    pub social_network: SocialNetworkEnum
}

pub struct TaskManager {
    pub task_queue: PriorityQueue<ParsingTask, Reverse<u64>>
}

impl TaskManager {

    pub fn new(setting: SettingsPtr) -> TaskManager {

        let mut parsing_tasks: Vec<ParsingTask> = Vec::new();

        for social_network_settings in setting.social_network_settings.iter() {

            let mut tasks = dispatch_social_network(
                &social_network_settings.parsing_tasks, 
                social_network_settings.social_network, 
                |parsing_tasks, social_network_ptr| social_network_ptr.process_settings_tasks(&parsing_tasks) )
                .expect("unable to process tasks from settings file");
            info!("{:#?}", tasks);
            parsing_tasks.append(&mut tasks);
        }

        let mut priority_queue: PriorityQueue<ParsingTask, Reverse<u64>> = PriorityQueue::with_capacity(parsing_tasks.len());

        for parsing_task in parsing_tasks.iter() {
            priority_queue.push(parsing_task.clone(), Reverse(parsing_task.execution_time));
        }

        return TaskManager { 
            task_queue: priority_queue
        }
    }

    pub fn add_parsing_task(&mut self, parsing_task: &ParsingTask) {
        self.task_queue
            .push(
                parsing_task.clone(), 
                Reverse(parsing_task.execution_time)
            );
    }

    pub fn get_parsing_task(&mut self) -> Option<ParsingTask> {
        match self.task_queue.pop() {
            Some((task, execution_time)) => Some(task),
            None => None,
        }
    }

    pub fn add_parsing_tasks(&mut self, tasks: Vec<ParsingTask>) {
        for task in tasks.iter() {
            self.add_parsing_task(&task);
        }
    }

    pub fn get_parsing_tasks(&mut self) -> Vec<ParsingTask> {
        let current_time = get_timestamp();
        let mut tasks: Vec<ParsingTask> = Vec::new();
        while let Some(task) = self.task_queue.peek() {
            if task.1.0 < current_time {
                tasks.push(self.task_queue.pop().unwrap().0);
            }
        }
        return tasks;
    }

    pub fn get_grouped_tasks(&mut self) -> HashMap<SocialNetworkEnum, Vec<ParsingTask>> {
        let mut hash_map: HashMap<SocialNetworkEnum, Vec<ParsingTask>> = HashMap::new();
        let mut parsing_tasks = self.get_parsing_tasks();
        for task in parsing_tasks.iter() {
            match hash_map.entry(task.social_network) {
                std::collections::hash_map::Entry::Occupied(mut v) => v.get_mut().push(task.clone()),
                std::collections::hash_map::Entry::Vacant(map) => {
                    map.insert(vec![task.clone()]);
                },
            }
        }
        return hash_map;
    }

    pub fn delete_task(&mut self, parsing_task: &ParsingTask) {
        self
            .task_queue
            .remove(parsing_task);
    }

    pub fn get_size(&self) -> usize {
        return self.task_queue.len();
    }

}