use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use strum::EnumIter;

use crate::{reddit::task_type::{RedditTaskType, RedditUrlWithPlaceholderSourceType}, client::db::client::{DBCollection, DATABASE_COLLECTIONS}};

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
    Reddit(RedditParsingParameters)
}

impl ParsingTaskParameters {
    pub fn as_reddit(self) -> RedditParsingParameters {
        return match self {
            ParsingTaskParameters::Reddit(reddit_parameters) => reddit_parameters,
            _ => panic!("wrong method dispatch")
        }
    }

    pub fn as_ref_reddit(&self) -> &RedditParsingParameters {
        return match self {
            ParsingTaskParameters::Reddit(reddit_parameters) => reddit_parameters,
            _ => panic!("wrong method dispatch")
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq, Debug)]
pub struct RedditParsingParameters {
    pub thread: String,
    pub reddit_task_type: RedditTaskType,
    pub after: Option<String>,
    pub id: Option<String>,
}

impl RedditParsingParameters {
    pub fn new(thread: String, reddit_task_type: RedditTaskType, after: Option<String>, id: Option<String>) -> ParsingTaskParameters {
        return ParsingTaskParameters::Reddit(RedditParsingParameters {
            thread,
            reddit_task_type,
            after,
            id
        });
    }

    pub fn to_url(&self) -> String {
        return match self.reddit_task_type.to_url().0 {
            RedditUrlWithPlaceholderSourceType::Thread(_) => self.reddit_task_type.to_url().to_string(self.thread.clone(), self.after.clone()),
            RedditUrlWithPlaceholderSourceType::Post(_) => self.reddit_task_type.to_url().to_string(self.thread.clone(), self.id.clone())
        };
    }
}