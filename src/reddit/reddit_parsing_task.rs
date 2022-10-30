use serde::{Serialize, Deserialize};
use strum::{EnumIter, EnumString, Display};

//rework this with macro

#[derive(Serialize, Deserialize, Display, Debug, Clone, Eq, Hash, PartialEq, EnumIter, EnumString)]
pub enum RedditParsingTask {
    ThreadNew{thread: String, after: Option<String>},
    ThreadTopAllTimeHistory{thread: String, after: Option<String>}, 
    ThreadTopYearHistory{thread: String, after: Option<String>},
    ThreadTopMonthHistory{thread: String, after: Option<String>},
    ThreadTopWeekHistory{thread: String, after: Option<String>},
    Post{thread: String, id: Option<String>, update_number: u64}
}

impl RedditParsingTask {

    pub fn get_all(thread: &String) -> Vec<RedditParsingTask> {
        return vec![
            RedditParsingTask::ThreadNew { thread: thread.clone(), after: None },
            RedditParsingTask::ThreadTopWeekHistory { thread: thread.clone(), after: None },
            RedditParsingTask::ThreadTopMonthHistory { thread: thread.clone(), after: None },
            RedditParsingTask::ThreadTopYearHistory { thread: thread.clone(), after: None },
            RedditParsingTask::ThreadTopAllTimeHistory { thread: thread.clone(), after: None }
        ];
    }

    pub fn to_url(&self) -> String {
        return match self {
            RedditParsingTask::Post{thread, id, .. } => 
                format!(
                    "https://oauth.reddit.com/{}/comments/{ID}?sort=top.json",
                    thread,
                    ID = id.as_ref().unwrap_or(&"".to_string()).strip_prefix("t3_").unwrap_or(&"")
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
            RedditParsingTask::ThreadNew { .. } => "new.json?",
            RedditParsingTask::ThreadTopAllTimeHistory { .. } => "top.json?t=all",
            RedditParsingTask::ThreadTopYearHistory { .. } => "top.json?t=year",
            RedditParsingTask::ThreadTopMonthHistory { .. } => "top.json?t=month",
            RedditParsingTask::ThreadTopWeekHistory { .. } => "top.json?t=week",
            RedditParsingTask::Post { .. } => panic!("this branch should not be reached"),
        };
        return format!("{}{}", base_rq, filter_rq);
    }

    fn get_after(&self) -> Option<String> {
        match self {
            RedditParsingTask::ThreadNew{ thread: _, after } |
            RedditParsingTask::ThreadTopAllTimeHistory{  thread: _, after } |
            RedditParsingTask::ThreadTopYearHistory{  thread: _, after } |
            RedditParsingTask::ThreadTopMonthHistory{  thread: _, after } |
            RedditParsingTask::ThreadTopWeekHistory {  thread: _, after } => after.clone(),
            RedditParsingTask::Post { .. } => panic!("this branch should not be reached"),
        }
    }

    pub fn get_thread(&self) -> String {
        return match self {
            RedditParsingTask::ThreadNew { thread, .. } |
            RedditParsingTask::ThreadTopAllTimeHistory { thread, .. } |
            RedditParsingTask::ThreadTopYearHistory { thread, .. } |
            RedditParsingTask::ThreadTopMonthHistory { thread, .. } |
            RedditParsingTask::ThreadTopWeekHistory { thread, .. } |
            RedditParsingTask::Post { thread, .. } => thread.clone(),
        }
    }

    pub fn with_parameter(&self, parameter: Option<String>) -> RedditParsingTask {
        return match self {
            RedditParsingTask::ThreadNew { thread, after: _ } => RedditParsingTask::ThreadNew { thread: thread.clone(), after: parameter },
            RedditParsingTask::ThreadTopAllTimeHistory { thread, after: _ } => RedditParsingTask::ThreadTopAllTimeHistory { thread: thread.clone(), after: parameter },
            RedditParsingTask::ThreadTopYearHistory { thread, after: _ } => RedditParsingTask::ThreadTopYearHistory { thread: thread.clone(), after: parameter },
            RedditParsingTask::ThreadTopMonthHistory { thread, after: _ } => RedditParsingTask::ThreadTopMonthHistory { thread: thread.clone(), after: parameter },
            RedditParsingTask::ThreadTopWeekHistory { thread, after: _ } => RedditParsingTask::ThreadTopWeekHistory { thread: thread.clone(), after: parameter },
            RedditParsingTask::Post { thread, id, update_number } => RedditParsingTask::Post { thread: thread.clone(), id: id.clone(), update_number: *update_number },
        }
    }
}