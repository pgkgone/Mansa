use crate::{generic::entity::Entity, client::managers::task_manager::ParsingTask};

use super::client::{DATABASE, DATABASE_COLLECTIONS, insert_if_not_empty};

pub async fn insert_entities(entities: &Vec<Entity>) {
    insert_if_not_empty::<Entity>(entities, DATABASE::MANSA, DATABASE_COLLECTIONS::ENTITIES).await;
}