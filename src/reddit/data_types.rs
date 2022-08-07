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

impl RedditTaskType {
    pub fn to_url(&self) -> RedditUrlWithPlaceholders {
        match self {
            RedditTaskType::All => {panic!("can't apply to_url to that variant");}
            RedditTaskType::ThreadNew => RedditUrlWithPlaceholders(RedditUrlWithPlaceholderSourceType::Thread("https://oauth.reddit.com/{THREAD}/new.json?{AFTER}".to_string())),
            RedditTaskType::ThreadTopAllTimeHistory => RedditUrlWithPlaceholders(RedditUrlWithPlaceholderSourceType::Thread("https://oauth.reddit.com/{THREAD}/top.json?t=all{AFTER}".to_string())),
            RedditTaskType::ThreadTopYearHistory => RedditUrlWithPlaceholders(RedditUrlWithPlaceholderSourceType::Thread("https://oauth.reddit.com/{THREAD}/top.json?t=year{AFTER}".to_string())),
            RedditTaskType::ThreadTopMonthHistory => RedditUrlWithPlaceholders(RedditUrlWithPlaceholderSourceType::Thread("https://oauth.reddit.com/{THREAD}/top.json?t=month{AFTER}".to_string())),
            RedditTaskType::ThreadTopWeekHistory => RedditUrlWithPlaceholders(RedditUrlWithPlaceholderSourceType::Thread("https://oauth.reddit.com/{THREAD}/top.json?t=week{AFTER}".to_string())),
            RedditTaskType::Post => RedditUrlWithPlaceholders(RedditUrlWithPlaceholderSourceType::Post("https://www.oauth.reddit.com/{THREAD}/comments/{ID}.json".to_string())),
        }
    }
}

pub struct RedditUrlWithPlaceholders(pub RedditUrlWithPlaceholderSourceType);

pub enum RedditUrlWithPlaceholderSourceType {
    Thread(String),
    Post(String)
}

impl RedditUrlWithPlaceholders {
    pub fn to_string(&self, thread: String, parameter: Option<String>) -> String {
        return match &self.0 {
            RedditUrlWithPlaceholderSourceType::Thread(template) => template.replace("{THREAD}", &thread).replace("{AFTER}", parameter.as_ref().map_or("".to_string(), |after| format!("&after={}&limit=100", &after)).as_str()),
            RedditUrlWithPlaceholderSourceType::Post(template) => template.replace("{THREAD}", &thread).replace("{ID}", parameter.as_ref().map_or("".to_string(), |id| format!("&after={}&limit=100", &id)).as_str()),
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Thread {
    pub kind: Option<String>, 
    pub data: ThreadMeta
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ThreadMeta {
    pub after: Option<String> ,
    pub children: Vec<ChildrenMeta>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChildrenMeta {
    pub kind: Option<String>, 
    pub data: Children
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

pub struct RedditParsingProps {
    pub task_type: RedditTaskType,
    pub after: Option<String>
}