use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> u64 {
    return SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
}