use std::error;

use log::info;
use serde::{Serialize, Deserialize};
use strum::{EnumIter, EnumString, Display};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq, EnumIter, Display, EnumString)]
pub enum RedditTaskType {
    All,
    ThreadNew,
    ThreadTopAllTimeHistory, 
    ThreadTopYearHistory,
    ThreadTopMonthHistory,
    ThreadTopWeekHistory,
    Post
}

pub struct RedditUrlWithPlaceholders(pub String);

impl RedditUrlWithPlaceholders {

    //&after=t3_p2ydga
    pub fn to_string(&self, thread: String, after: Option<String>) -> String {
        let r = self.0.replace("{THREAD}", &thread);
        return  r.replace("{AFTER}", &after.map_or("".to_string(), |a| format!("&after={}&limit=100", &a)));
    }

    pub fn reddit_task_type_to_string(task_type: RedditTaskType) -> RedditUrlWithPlaceholders {
        info!("{}", task_type);
        match task_type {
            RedditTaskType::ThreadNew => RedditUrlWithPlaceholders("https://oauth.reddit.com/{THREAD}/new.json?{AFTER}".to_string()),
            RedditTaskType::ThreadTopAllTimeHistory => RedditUrlWithPlaceholders("https://oauth.reddit.com/{THREAD}/top.json?t=all{AFTER}".to_string()),
            RedditTaskType::ThreadTopYearHistory => RedditUrlWithPlaceholders("https://oauth.reddit.com/{THREAD}/top.json?t=year{AFTER}".to_string()),
            RedditTaskType::ThreadTopMonthHistory => RedditUrlWithPlaceholders("https://oauth.reddit.com/{THREAD}/top.json?t=month{AFTER}".to_string()),
            RedditTaskType::ThreadTopWeekHistory => RedditUrlWithPlaceholders("https://oauth.reddit.com/{THREAD}/top.json?t=week{AFTER}".to_string()),
            RedditTaskType::Post => todo!(),
            _ => todo!()
        }
    }
    
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Thread {
    pub kind: Option<String>, 
    pub data: ThreadMeta
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ThreadMeta {
    pub after: Option<String> ,
    pub children: Vec<ChildrenMeta>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChildrenMeta {
    pub kind: Option<String>, 
    pub data: Children
}

#[derive(Serialize, Deserialize, Clone)]
pub struct  Children {
    pub kind: Option<String>, 
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Preview {
    pub images: Vec<Image>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Image {
    pub source: Source
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Source {
    pub url: String
}