use std::{path::{Path}, fs, env};
use log::error;
use serde::de::DeserializeOwned;
use serde_json;

pub fn read_json_from_file<T: DeserializeOwned>(path: &Path) -> T {
    let file_content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => {
            error!("current path: {}", env::current_dir().unwrap().as_path().to_str().unwrap());
            error!("target path: {}", path.to_str().unwrap());
            panic!("unable to find file.")
        }
    };
    
    let settings =  match serde_json::from_str(&file_content) {
        Ok(content) => content, 
        Err(err) => {
            panic!("unable to parse file content. {}", err); 
        }
    };

    return settings;
}