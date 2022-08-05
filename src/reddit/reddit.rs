use std::{error::Error, str::FromStr, collections::HashMap};

use async_trait::async_trait;
use chrono::DateTime;
use futures::future::try_join_all;
use log::{error, info};
use regex::Regex;
use reqwest::{Response, StatusCode};
use lazy_static::{lazy_static};
use crate::{generic::{social_network::{SocialNetwork, SocialNetworkEnum}, entity::Entity}, client::{http_client::HttpAuthData, settings::ParsingTaskSettings, parser::AccountManagerPtr, db::{entities_db}, managers::{account_manager::{AccountPtr, ReqwestClientPtr}, task_manager::{ParsingTask, ParsingTaskStatus}}}, utils::time::get_timestamp};

use super::data_types::{AuthResponse, Thread, RedditTaskType, RedditUrlWithPlaceholders};

pub struct Reddit {
    pub auth_url: String
}

impl Default for Reddit {
    fn default() -> Self {
        Self { 
            auth_url: String::from("https://www.reddit.com/api/v1/access_token") 
        }
    }
}

unsafe impl Sync for Reddit {}

#[async_trait]
impl SocialNetwork for Reddit {
    
    async fn auth(
        &self, 
        account_ptr: AccountPtr, 
        client_ptr: ReqwestClientPtr
    ) -> Result<HttpAuthData, Box<dyn Error + Send + Sync>> {
        let mut response = client_ptr
            .post(self.auth_url.clone())
            .basic_auth(
                account_ptr.public_key.as_ref().ok_or("no public key")?, 
                account_ptr.private_key.as_ref()
            )
            .body(format!(
                "grant_type=password&username={}&password={}", 
                account_ptr.login.as_ref().ok_or("no login")?,
                account_ptr.password.as_ref().ok_or("no password")?
            ))
            .send()
            .await?;
        let (response_timestamp, millis_to_refresh, requests_limit) = Reddit::parse_limits_from_header(&response);
        let auth_response_json = response.json::<AuthResponse>()
            .await?;
        return Ok(HttpAuthData{ 
            token: auth_response_json.access_token, 
            retrieve_timestamp: get_timestamp(), 
            millis_to_refresh: millis_to_refresh, 
            requests_limit: requests_limit 
        });
    }

    fn process_settings_tasks(&self, tasks: &Vec<ParsingTaskSettings>) -> Result<Vec<ParsingTask>, Box<dyn Error>> {
        let mut processed_settings_tasks: Vec<ParsingTask> = Vec::new();
        for settings_task in tasks.iter() {
            let reddit_task_type = RedditTaskType::from_str(&settings_task.task_type.clone()).expect("unable to convert settings task type to RedditTaskType");
            let s = RedditUrlWithPlaceholders::reddit_task_type_to_string(reddit_task_type).to_string(settings_task.props.get("thread").expect("unable to find thread prop").to_string(), None);
            processed_settings_tasks.push(
                ParsingTask { 
                    _id: None,
                    execution_time: get_timestamp(), 
                    url: s, 
                    action_type: settings_task.task_type.clone(), 
                    social_network: SocialNetworkEnum::Reddit,
                    status: ParsingTaskStatus::New
                }
            );
        }
        return Ok(processed_settings_tasks);
    }

    async fn parse(&self, account_manager_ptr: AccountManagerPtr, account: (AccountPtr, HttpAuthData), parsing_task: Vec<ParsingTask>) -> (Option<HttpAuthData>, Vec<ParsingTask>, Vec<ParsingTask>) {

        info!("parsing reddit task");
        //locking
        let mut account_manager_lock = account_manager_ptr.write().await;
        let client = account_manager_lock.get_client(account.0.clone()).unwrap().clone();
        drop(account_manager_lock);

        let mut requests_map: HashMap<String, &ParsingTask> = HashMap::new();

        let requests = parsing_task.iter().map(|task| {

            requests_map.insert(task.url.clone(), task);

            return client.clone()
                .get(task.url.clone())
                .bearer_auth(account.1.token.clone())
                .send()
        });

        let responses = try_join_all(requests).await;
        
        if responses.is_err() {
            return (None, Vec::new(), Vec::new());
        }

        let mut new_parsing_tasks: Vec<ParsingTask> = Vec::new();
        let mut errored_parsing_tasks: Vec<ParsingTask> = Vec::new();
        let mut auth_data: HttpAuthData = account.1.clone();

        for response in responses.unwrap() {
            let (response_timestamp, millis_to_refresh, requests_limit) = Reddit::parse_limits_from_header(&response);
            let correspond_parsing_task = (**requests_map.get(&response.url().to_string()).unwrap()).clone();
            //why it's not a enum??? reqwest WTF?
            if response.status() == StatusCode::OK {
                let response_body = response.json::<Thread>().await.inspect_err(|err| error!("account: {:?}, error: {}", account.0.clone(), err));
                if response_body.is_ok() {
                    let thread = response_body.unwrap();
                    new_parsing_tasks.extend(Reddit::spawn_new_tasks(&correspond_parsing_task, &thread));
                    entities_db::insert_entities(&Self::get_entities(&thread)).await;
                }
            } else {
                errored_parsing_tasks.push(correspond_parsing_task);
                if response.status() == StatusCode::FORBIDDEN {
                    error!("TO DO: re auth if http 403!");
                }
            }
            if(auth_data.retrieve_timestamp < response_timestamp) {
                auth_data = HttpAuthData{ 
                    token: auth_data.token.clone(), 
                    retrieve_timestamp: response_timestamp, 
                    millis_to_refresh, 
                    requests_limit 
                }
            }

        }
        return (Some(auth_data), new_parsing_tasks, errored_parsing_tasks);
    }
}

impl Reddit {
    fn parse_limits_from_header(response: &Response) -> (u64, u64, usize) {
        //Sun, 31 Jul 2022 00:01:30 GMT
        let timestamp: u64 = response
            .headers()
            .get("Date")
            .and_then(|date| DateTime::parse_from_rfc2822(date.to_str().unwrap()).map(|date| date.timestamp() as u64).ok())
            .unwrap_or(0);

        let millis_to_refresh: u64 = response
            .headers()
            .get("x-ratelimit-reset")
            .map(|v| v.to_str().unwrap())
            .unwrap_or("400")
            .parse().unwrap_or(400);
            
        let requests_limit: usize = response
            .headers()
            .get("x-ratelimit-remaining")
            .map(|v| v.to_str().unwrap())
            .unwrap_or("0")
            .parse().unwrap_or(0);
        return (timestamp, millis_to_refresh, requests_limit);
    }

    fn spawn_new_tasks(parsing_task: &ParsingTask, thread: &Thread) -> Vec<ParsingTask> {
        let mut parsing_tasks: Vec<ParsingTask> = Vec::new();
        let after = thread.data.after.clone();
        if after.is_some() {
            parsing_tasks.push(ParsingTask {
                _id: None,
                execution_time: get_timestamp(),
                url: RedditUrlWithPlaceholders::reddit_task_type_to_string(RedditTaskType::from_str(&parsing_task.action_type).unwrap()).to_string(Self::get_thread_from_url(&parsing_task.url) , after),
                action_type: parsing_task.action_type.clone(),
                social_network: SocialNetworkEnum::Reddit,
                status: ParsingTaskStatus::New
            })
        }
        return parsing_tasks;
    }

    fn get_thread_from_url(url: &String) -> String {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(r/)(.*?)(/)").unwrap();
        }
        let c = RE.captures_iter(url).next().expect("url parsing error!!!!");
        return format!("{}{}", &c[1], &c[2])
    }

    fn get_entities(thread: &Thread) -> Vec<Entity> {
        let mut entities: Vec<Entity> = Vec::new();
        for post in thread.data.children.iter().map(|v| &v.data) {
            entities.push(post.into());
        }
        return entities;
    }
}
