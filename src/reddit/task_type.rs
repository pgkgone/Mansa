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
