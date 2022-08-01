use crate::{generic::entity::Entity, client::managers::task_manager::ParsingTask};

use super::client::{DATABASE, DATABASE_COLLECTIONS, insert_if_not_empty};

pub async fn insert_tasks(tasks: &Vec<ParsingTask>) {
    insert_if_not_empty::<ParsingTask>(tasks, DATABASE::MANSA, DATABASE_COLLECTIONS::PARSING_TASKS).await;
}
