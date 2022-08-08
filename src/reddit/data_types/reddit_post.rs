use mongodb::bson::DateTime;
use serde::{Serialize, Deserialize};

use crate::generic::{entity::{Entity, EntityType}, social_network::SocialNetworkEnum};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct  Post {
    #[serde(rename = "name")]
    pub id: String,
    #[serde(rename = "created")] 
    pub timestamp: Option<f64>,
    #[serde(rename = "subreddit_name_prefixed")]
    pub source: Option<String>,
    #[serde(rename = "subreddit_subscribers")]
    pub source_followers: Option<u64>,
    pub title: String, 
    #[serde(alias = "selftext")]
    pub self_text: Option<String>, 
    #[serde(alias = "author_fullname")]
    pub author_id: Option<String>,
    #[serde(alias = "author")]
    pub author_name: Option<String>, 
    pub ups: u64,
    pub preview: Option<Preview>

}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Preview {
    pub images: Vec<Image>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Image {
    pub source: Source
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Source {
    pub url: String
}

impl From<Post> for Entity {
    fn from(children: Post) -> Self {
        return Entity {
            _id: None,
            id: children.id,
            source: children.source.unwrap_or(String::from("")),
            source_followers: children.source_followers,
            date_time: DateTime::from_millis(children.timestamp.unwrap_or(0.0) as i64 * 1000),
            entity_type: EntityType::Post, 
            author_id: children.author_id, 
            title: Some(children.title), 
            content: children.self_text, 
            author_name: children.author_name, 
            rating: Some(children.ups), 
            images: children
                .preview
                .map_or(Vec::new(), |v| v.images)
                .into_iter()
                .map(|v| v.source.url)
                .collect(),
            social_network: SocialNetworkEnum::Reddit,
        }
    }
}
