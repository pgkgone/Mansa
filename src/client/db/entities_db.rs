use std::thread;

use futures::future::join_all;
use log::info;
use mongodb::{bson::{doc, self}, options::UpdateOptions};

use crate::commons::entity::Entity;

use super::client::{DATABASE, DATABASE_COLLECTIONS, insert_if_not_empty, ENTITY_COLLECTION};

pub async fn insert_entities(entities: &Vec<Entity>) {
    insert_if_not_empty::<Entity>(entities, DATABASE::MANSA, DATABASE_COLLECTIONS::ENTITIES).await;
}

pub async fn insert_with_replace(entities: Vec<Entity>) {
    info!("2 {:?}", thread::current().id());
    let mut handlers = Vec::new();
    for item in entities.into_iter() {
        let options = UpdateOptions::builder()
            .upsert(Some(true))
            .build();
        let match_query = doc! {
            "id" : {
                "$eq" : item.id.clone()
            }
        };
        let d = bson::to_document(&item).unwrap();
        let update_query = doc! {
            "$set": d
        };
        handlers.push(
            tokio::spawn(ENTITY_COLLECTION.get().await.update_one(match_query, update_query, options))
        );
    }
    join_all(handlers).await;
}