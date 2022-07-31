use std::cmp::Ordering;

use derivative::Derivative;

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq, Hash, Eq)]
pub struct HttpAuthData {
    pub token: String,
    pub retrieve_timestamp: u64,
    pub millis_to_refresh: u64,
    pub requests_limit: usize,
}

impl Ord for HttpAuthData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        return match self.requests_limit.cmp(&other.requests_limit) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => (self.retrieve_timestamp + self.millis_to_refresh).cmp(&(other.retrieve_timestamp + other.millis_to_refresh)),
            Ordering::Greater => Ordering::Greater,
        };
    }
}

impl PartialOrd for HttpAuthData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return Some(self.cmp(other));
    }
}