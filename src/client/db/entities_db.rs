use std::{sync::Arc, thread};

use futures::future::join_all;
use log::info;
use mongodb::{bson::{doc, Document, self}, options::{UpdateOptions, FindOneAndUpdateOptions}};

use crate::{generic::entity::{Entity, self}, client::managers::task_manager::ParsingTask};

use super::client::{DATABASE, DATABASE_COLLECTIONS, insert_if_not_empty, get_collection, ENTITY_COLLECTION};

pub async fn insert_entities(entities: &Vec<Entity>) {
    insert_if_not_empty::<Entity>(entities, DATABASE::MANSA, DATABASE_COLLECTIONS::ENTITIES).await;
}

pub async fn insert_with_replace(mut entities: Vec<Entity>) {
    info!("2 {:?}", thread::current().id());
    let mut handlers = Vec::new();
    for item in entities.into_iter() {
        let options = UpdateOptions::builder()
            .upsert(Some(true))
            .build();
        let match_query = doc! {
            "_id" : {
                "$eq" : item._id.clone()
            }
        };
        let mut d = bson::to_document(&item).unwrap();
        d.remove("_id").unwrap();
        let update_query = doc! {
            "$set": d
        };
        handlers.push(
            tokio::spawn(ENTITY_COLLECTION.get().await.update_one(match_query, update_query, options))
        );
    }
    join_all(handlers).await;
}