use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Serialize, Deserialize};

use crate::{client::db::client::{DBCollection, DATABASE_COLLECTIONS}};

use super::social_network::SocialNetworkEnum;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EntityType {
    Post,
    Comment,
    Message
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub entity_type: EntityType,
    pub date_time: DateTime,
    //social network id
    pub id: String,
    pub source: String, 
    pub source_followers: Option<u64>,
    //social network author id
    pub author_id: Option<String>,

    pub title: Option<String>,
    pub content: Option<String>,
    pub author_name: Option<String>,
    pub social_network: SocialNetworkEnum,

    pub rating: Option<i64>, 

    pub images: Vec<String>
}

impl DBCollection for Entity {
    fn get_collection() -> String {
        return DATABASE_COLLECTIONS::ENTITIES.to_string();
    }
}