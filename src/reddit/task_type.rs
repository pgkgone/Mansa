use serde::{Serialize, Deserialize};
use strum::{EnumIter, EnumString, Display};

//rework this with macro

#[derive(Serialize, Deserialize, Display, Debug, Clone, Eq, Hash, PartialEq, EnumIter, EnumString)]
pub enum RedditTaskType {
    ThreadNew{thread: String, after: Option<String>},
    ThreadTopAllTimeHistory{thread: String, after: Option<String>}, 
    ThreadTopYearHistory{thread: String, after: Option<String>},
    ThreadTopMonthHistory{thread: String, after: Option<String>},
    ThreadTopWeekHistory{thread: String, after: Option<String>},
    Post{thread: String, id: Option<String>, update_number: u64}
}

impl RedditTaskType {

    pub fn get_vec_of_threads(thread: &String) -> Vec<RedditTaskType> {
        return vec![
            RedditTaskType::ThreadNew { thread: thread.clone(), after: None },
            RedditTaskType::ThreadTopWeekHistory { thread: thread.clone(), after: None },
            RedditTaskType::ThreadTopMonthHistory { thread: thread.clone(), after: None },
            RedditTaskType::ThreadTopYearHistory { thread: thread.clone(), after: None },
            RedditTaskType::ThreadTopAllTimeHistory { thread: thread.clone(), after: None }
        ];
    }

    pub fn to_url(&self) -> String {
        return match self {
            RedditTaskType::Post{thread, id, .. } => 
                format!(
                    "https://oauth.reddit.com/{}/comments/?sort=$old&threded={ID}.json", 
                    thread,
                    ID = id.as_ref().unwrap_or(&"".to_string()).as_str()
                ),
            _ => format!(
                "https://oauth.reddit.com/{}/{}", 
                self.get_thread(),
                self.get_url_request_string()
            ) 

        }
    } 

    fn get_url_request_string(&self) -> String {
        let filter_rq = self.get_after()
            .as_ref()
            .map_or("".to_string(), |after| format!("&after={}&limit=100", &after));
        let base_rq = match self {
            RedditTaskType::ThreadNew { .. } => "new.json?",
            RedditTaskType::ThreadTopAllTimeHistory { .. } => "top.json?t=all",
            RedditTaskType::ThreadTopYearHistory { .. } => "top.json?t=year",
            RedditTaskType::ThreadTopMonthHistory { .. } => "top.json?t=month",
            RedditTaskType::ThreadTopWeekHistory { .. } => "top.json?t=week",
            RedditTaskType::Post { .. } => panic!("this branch should not be reached"),
        };
        return format!("{}{}", base_rq, filter_rq);
    }

    fn get_after(&self) -> Option<String> {
        match self {
            RedditTaskType::ThreadNew{ thread, after } |
            RedditTaskType::ThreadTopAllTimeHistory{  thread, after } |
            RedditTaskType::ThreadTopYearHistory{  thread, after } |
            RedditTaskType::ThreadTopMonthHistory{  thread, after } |
            RedditTaskType::ThreadTopWeekHistory {  thread, after } => after.clone(),
            RedditTaskType::Post { .. } => panic!("this branch should not be reached"),
        }
    }

    pub fn get_thread(&self) -> String {
        return match self {
            RedditTaskType::ThreadNew { thread, .. } |
            RedditTaskType::ThreadTopAllTimeHistory { thread, .. } |
            RedditTaskType::ThreadTopYearHistory { thread, .. } |
            RedditTaskType::ThreadTopMonthHistory { thread, .. } |
            RedditTaskType::ThreadTopWeekHistory { thread, .. } |
            RedditTaskType::Post { thread, .. } => thread.clone(),
        }
    }

    pub fn with_parameter(&self, parameter: Option<String>) -> RedditTaskType {
        return match self {
            RedditTaskType::ThreadNew { thread, after: _ } => RedditTaskType::ThreadNew { thread: thread.clone(), after: parameter },
            RedditTaskType::ThreadTopAllTimeHistory { thread, after } => RedditTaskType::ThreadTopAllTimeHistory { thread: thread.clone(), after: parameter },
            RedditTaskType::ThreadTopYearHistory { thread, after } => RedditTaskType::ThreadTopYearHistory { thread: thread.clone(), after: parameter },
            RedditTaskType::ThreadTopMonthHistory { thread, after } => RedditTaskType::ThreadTopMonthHistory { thread: thread.clone(), after: parameter },
            RedditTaskType::ThreadTopWeekHistory { thread, after } => RedditTaskType::ThreadTopWeekHistory { thread: thread.clone(), after: parameter },
            RedditTaskType::Post { thread, id, update_number } => RedditTaskType::Post { thread: thread.clone(), id: id.clone(), update_number: *update_number },
        }
    }
}