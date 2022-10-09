use serde::{Serialize, Deserialize};
use strum::{EnumIter, EnumString, Display};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, Hash, PartialEq, EnumIter, EnumString)]
pub enum RedditTaskType {
    All{thread: String},
    ThreadNew{thread: String, after: Option<String>},
    ThreadTopAllTimeHistory{thread: String, after: Option<String>}, 
    ThreadTopYearHistory{thread: String, after: Option<String>},
    ThreadTopMonthHistory{thread: String, after: Option<String>},
    ThreadTopWeekHistory{thread: String, after: Option<String>},
    Post{thread: String, id: Option<String>, update_number: u64}
}

impl ToString for RedditTaskType {
    fn to_string(&self) -> String {
        return match self {
            RedditTaskType::All {..} => "All".to_string(),
            RedditTaskType::ThreadNew {..} => "ThreadNew".to_string(),
            RedditTaskType::ThreadTopAllTimeHistory {..} => "ThreadTopAllTimeHistory".to_string(),
            RedditTaskType::ThreadTopYearHistory {..} => "ThreadTopYearHistory".to_string(),
            RedditTaskType::ThreadTopMonthHistory {..} => "ThreadTopMonthHistory".to_string(),
            RedditTaskType::ThreadTopWeekHistory {..} => "ThreadTopWeekHistory".to_string(),
            RedditTaskType::Post {..} => "Post".to_string(),
        }
    }
}

impl RedditTaskType {
    pub fn to_url(&self) -> String {
        return match self {
            RedditTaskType::All{thread} => {panic!("can't ToString this variant");}
            RedditTaskType::ThreadNew{thread, after} => format!("https://oauth.reddit.com/{}/new.json?{{AFTER}}", thread)
                .replace("{AFTER}", after.as_ref().map_or("".to_string(), |after| format!("&after={}&limit=100", &after)).as_str()),
            RedditTaskType::ThreadTopAllTimeHistory{thread, after} => format!("https://oauth.reddit.com/{}/top.json?t=all{{AFTER}}", thread)
                .replace("{AFTER}", after.as_ref().map_or("".to_string(), |after| format!("&after={}&limit=100", &after)).as_str()),
            RedditTaskType::ThreadTopYearHistory{thread, after} => format!("https://oauth.reddit.com/{}/top.json?t=year{{AFTER}}", thread)
                .replace("{AFTER}", after.as_ref().map_or("".to_string(), |after| format!("&after={}&limit=100", &after)).as_str()),
            RedditTaskType::ThreadTopMonthHistory{thread, after} => format!("https://oauth.reddit.com/{}/top.json?t=month{{AFTER}}", thread)
                .replace("{AFTER}", after.as_ref().map_or("".to_string(), |after| format!("&after={}&limit=100", &after)).as_str()),
            RedditTaskType::ThreadTopWeekHistory{thread, after} => format!("https://oauth.reddit.com/{}/top.json?t=week{{AFTER}}", thread)
                .replace("{AFTER}", after.as_ref().map_or("".to_string(), |after| format!("&after={}&limit=100", &after)).as_str()),
            RedditTaskType::Post{thread, id, update_number} => format!("https://oauth.reddit.com/{}/comments/?sort=$old&threded={ID}.json", thread,
                ID = id.as_ref().unwrap_or(&"".to_string()).as_str()),
        }
    } 

    pub fn get_thread(&self) -> String {
        return match self {
            RedditTaskType::All { thread } => thread.clone(),
            RedditTaskType::ThreadNew { thread, after } => thread.clone(),
            RedditTaskType::ThreadTopAllTimeHistory { thread, after } => thread.clone(),
            RedditTaskType::ThreadTopYearHistory { thread, after } => thread.clone(),
            RedditTaskType::ThreadTopMonthHistory { thread, after } => thread.clone(),
            RedditTaskType::ThreadTopWeekHistory { thread, after } => thread.clone(),
            RedditTaskType::Post { thread, id, update_number } => thread.clone(),
        }
    }

    pub fn with_parameter(&self, parameter: Option<String>) -> RedditTaskType {
        return match self {
            RedditTaskType::All { thread } => RedditTaskType::All{thread: thread.clone() },
            RedditTaskType::ThreadNew { thread, after } => RedditTaskType::ThreadNew { thread: thread.clone(), after: parameter },
            RedditTaskType::ThreadTopAllTimeHistory { thread, after } => RedditTaskType::ThreadTopAllTimeHistory { thread: thread.clone(), after: parameter },
            RedditTaskType::ThreadTopYearHistory { thread, after } => RedditTaskType::ThreadTopYearHistory { thread: thread.clone(), after: parameter },
            RedditTaskType::ThreadTopMonthHistory { thread, after } => RedditTaskType::ThreadTopMonthHistory { thread: thread.clone(), after: parameter },
            RedditTaskType::ThreadTopWeekHistory { thread, after } => RedditTaskType::ThreadTopWeekHistory { thread: thread.clone(), after: parameter },
            RedditTaskType::Post { thread, id, update_number } => RedditTaskType::Post { thread: thread.clone(), id: id.clone(), update_number: *update_number },
        }
    }
}