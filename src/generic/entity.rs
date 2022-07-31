use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Serialize, Deserialize};

use crate::reddit::data_types::{Children};

use super::social_network::SocialNetworkEnum;
#[derive(Debug, Serialize, Deserialize)]
enum EntityType {
    Post,
    Message
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    #[serde(skip_serializing_if = "Option::is_none")]
    _id: Option<ObjectId>,
    entity_type: EntityType,
    date_time: DateTime,
    //social network id
    id: String,
    source: String, 
    source_followers: u64,
    //social network author id
    author_id: Option<String>,

    title: Option<String>,
    content: String,
    author_name: String,
    social_network: SocialNetworkEnum,

    rating: Option<u64>, 

    images: Vec<String>
}

impl From<&Children> for Entity {
    fn from(children: &Children) -> Self {
        return Entity { 
            _id: None,
            source: children.source.clone().unwrap_or("".to_string()),
            source_followers: children.source_followers.unwrap_or(0),
            date_time: DateTime::from_millis(children.timestamp.unwrap_or(0.0) as i64),
            entity_type: EntityType::Post, 
            id: children.id.clone(), 
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