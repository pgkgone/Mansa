use super::reddit_pages::{*, self};

pub enum ResponseBody {
    Thread(Result<reddit_pages::ThreadPage, reqwest::Error>),
    Comments(Result<reddit_pages::CommentPage, reqwest::Error>)
}

impl ResponseBody {
    pub fn is_ok(&self) -> bool {
        return match self {
            ResponseBody::Thread(thread) => thread.is_ok(),
            ResponseBody::Comments(comments) => comments.is_ok(),
        }
    }
}