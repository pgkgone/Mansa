use futures::future::join_all;
use mongodb::{bson::{doc, Document, self}, options::{UpdateOptions, FindOneAndUpdateOptions}};

use crate::{generic::entity::{Entity, self}, client::managers::task_manager::ParsingTask};

use super::client::{DATABASE, DATABASE_COLLECTIONS, insert_if_not_empty, get_collection};

pub async fn insert_entities(entities: &Vec<Entity>) {
    insert_if_not_empty::<Entity>(entities, DATABASE::MANSA, DATABASE_COLLECTIONS::ENTITIES).await;
}

pub async fn insert_with_replace(entities: &Vec<Entity>) {
    let collection = get_collection::<Entity>().await;
    let mut futures = Vec::new();
    for item in entities {
        let options = UpdateOptions::builder()
            .upsert(Some(true))
            .build();
        let match_query = doc! {
            "_id" : {
                "$eq" : item._id.clone()
            }
        };
        let mut d = bson::to_document(item).unwrap();
        d.remove("_id").unwrap();
        let update_query = doc! {
            "$set": d
        };
        futures.push(collection.update_one(match_query, update_query, options));
    }
    join_all(futures).await;
}