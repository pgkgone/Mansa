use mongodb::bson::DateTime;
use serde::{Serialize, Deserialize};

use crate::generic::{entity::{Entity, EntityType}, social_network::SocialNetworkEnum};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Comment {
    pub id: String,
    pub parent_id: String,
    #[serde(rename = "created")] 
    pub timestamp: Option<f64>,
    pub score: u64, 
    #[serde(alias = "author_fullname")]
    pub author_id: Option<String>,
    #[serde(alias = "author")]
    pub author_name: Option<String>, 
    #[serde(rename = "subreddit_name_prefixed")]
    pub source: Option<String>,
    pub body: Option<String>, 
}

impl From<Comment> for Entity {
    fn from(comment: Comment) -> Self {
        return Entity {
            _id: None,
            id: comment.id,
            source: comment.source.unwrap_or("".to_string()),
            source_followers: None,
            date_time: DateTime::from_millis(comment.timestamp.unwrap_or(0.0) as i64 * 1000),
            entity_type: EntityType::Comment,
            author_id: comment.author_id, 
            title: None, 
            content: comment.body, 
            author_name: comment.author_name, 
            rating: Some(comment.score), 
            images: Vec::new(),
            social_network: SocialNetworkEnum::Reddit,
        }
    }
}