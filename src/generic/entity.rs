use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Serialize, Deserialize};

use crate::{reddit::data_types::{Children}, client::db::client::{DBCollection, DATABASE_COLLECTIONS}};

use super::social_network::SocialNetworkEnum;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EntityType {
    Post,
    Message
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entity {
    pub entity_type: EntityType,
    pub date_time: DateTime,
    //social network id
    pub _id: String,
    pub source: String, 
    pub source_followers: u64,
    //social network author id
    pub author_id: Option<String>,

    pub title: Option<String>,
    pub content: String,
    pub author_name: String,
    pub social_network: SocialNetworkEnum,

    pub rating: Option<u64>, 

    pub images: Vec<String>
}

impl DBCollection for Entity {
    fn get_collection() -> String {
        return DATABASE_COLLECTIONS::ENTITIES.to_string();
    }
}

impl From<Children> for Entity {
    fn from(children: Children) -> Self {
        return Entity {
            _id: children.id.clone(), 
            source: children.source.clone().unwrap_or("".to_string()),
            source_followers: children.source_followers.unwrap_or(0),
            date_time: DateTime::from_millis(children.timestamp.unwrap_or(0.0) as i64 * 1000),
            entity_type: EntityType::Post, 
            author_id: Some(children.author_id.as_ref().unwrap_or(&"".to_string()).clone()), 
            title: Some(children.title.clone()), 
            content: children.self_text.clone().unwrap_or(String::from("")), 
            author_name: children.author_name.as_ref().unwrap_or(&"".to_string()).clone(), 
            rating: Some(children.ups), 
            images: children
                .preview
                .clone()
                .map_or(Vec::new(), |v| v.images)
                .iter()
                .map(|v| v.source.url.clone())
                .collect(),
            social_network: SocialNetworkEnum::Reddit,


            
        }
    }
}