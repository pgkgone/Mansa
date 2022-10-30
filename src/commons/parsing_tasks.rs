use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use strum::EnumIter;

use crate::{reddit::reddit_parsing_task::{RedditParsingTask}, client::db::client::{DBCollection, DATABASE_COLLECTIONS}};

use super::social_network::SocialNetworkEnum;

#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq, Debug, EnumIter, strum::Display)]
pub enum ParsingTaskStatus {
    New, 
    Processing,
    Processed
}

#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ParsingTask {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub execution_time: u64,
    pub parameters: ParsingTaskParameters,
    pub action_type: String,
    pub social_network: SocialNetworkEnum,
    pub status: ParsingTaskStatus
}

impl DBCollection for ParsingTask {
    fn get_collection() -> String {
        return DATABASE_COLLECTIONS::PARSING_TASKS.to_string();
    }
}

#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq, Debug)]
pub enum ParsingTaskParameters {
    Reddit(RedditParsingTask)
}

impl ParsingTaskParameters {
    pub fn as_reddit(self) -> RedditParsingTask {
        return match self {
            ParsingTaskParameters::Reddit(reddit_parameters) => reddit_parameters,
            _ => panic!("wrong method dispatch")
        }
    }

    pub fn as_ref_reddit(&self) -> &RedditParsingTask {
        return match self {
            ParsingTaskParameters::Reddit(reddit_parameters) => reddit_parameters,
            _ => panic!("wrong method dispatch")
        }
    }
}