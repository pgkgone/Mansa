use serde::{Serialize, Deserialize, Deserializer};
use serde_json::Value;

use super::{reddit_listing::Listing, reddit_comment::Comment, reddit_post::Post};

#[derive(Serialize, Clone, Debug)]
pub struct ThreadPage {
    pub posts: Listing<Post>
}

#[derive(Serialize, Clone, Debug)]
pub struct CommentPage {
    pub post: Listing<Post>,
    pub comments: Listing<Comment>
}

impl<'de> Deserialize<'de> for ThreadPage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let listing: Listing<Post> = Deserialize::deserialize(deserializer)?;
        return Ok(ThreadPage{ posts: listing });
    }
}

impl<'de> Deserialize<'de> for CommentPage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let listings: Vec<Value> = Deserialize::deserialize(deserializer)?;
        let children: Listing<Post> = Deserialize::deserialize(listings
            .get(0)
            .unwrap()
        ).expect("parsing error");
        let comments: Listing<Comment> = Deserialize::deserialize(listings
            .get(1)
            .unwrap()
        ).expect("parsing error");
        return Ok(CommentPage{ post: children, comments });
    }
}