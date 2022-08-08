use std::{error::Error};

use async_trait::async_trait;
use chrono::DateTime;
use futures::{future::{try_join_all}, FutureExt};
use log::{error, info};
use reqwest::{Response, StatusCode};
use crate::{generic::{social_network::{SocialNetwork, SocialNetworkEnum}, entity::Entity, parsing_tasks::{RedditParsingParameters, ParsingTask, ParsingTaskParameters, ParsingTaskStatus}}, client::{http_client::HttpAuthData, parser::AccountManagerPtr, db::{entities_db::{insert_with_replace}, tasks_db::update_tasks_with_status}, managers::{account_manager::{AccountPtr, ReqwestClientPtr}}}, utils::time::get_timestamp};
use strum::IntoEnumIterator;
use super::{task_type::{RedditTaskType}, data_types::{reddit_auth::AuthResponse, reddit_pages::ThreadPage}};
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

    fn process_settings_tasks(&self, tasks: &Vec<ParsingTaskParameters>) -> Result<Vec<ParsingTask>, Box<dyn Error>> {
        let mut parsing_tasks: Vec<ParsingTask> = Vec::new();
        for task in tasks {
            let reddit_parsing_parameters = match task {
                ParsingTaskParameters::Reddit(params) => params,
                _ => continue
            };
            match reddit_parsing_parameters.reddit_task_type {
                RedditTaskType::All => parsing_tasks.extend(Reddit::unfold_all(reddit_parsing_parameters)),
                RedditTaskType::Post => continue,
                _ => parsing_tasks.push(Reddit::create_parsing_task(reddit_parsing_parameters.clone()))
            }
        }
        return Ok(parsing_tasks);
    }

    async fn parse(&self, account_manager_ptr: AccountManagerPtr, account: (AccountPtr, HttpAuthData), parsing_task: Vec<ParsingTask>) -> (Option<HttpAuthData>, Vec<ParsingTask>) {
        info!("parsing reddit task");
        let mut account_manager_lock = account_manager_ptr.write().await;
        let client = account_manager_lock.get_client(account.0.clone()).unwrap().clone();
        drop(account_manager_lock);
        let requests = parsing_task.into_iter().map(
            move |task| {
                let token = account.1.token.clone();
                return tokio::spawn(client.clone()
                    .get(task.parameters.as_ref_reddit().to_url())
                    .bearer_auth(token.clone())
                    .send()
                    .then( 
                        move |response| Reddit::process_response(task, response, token)
                    )
                )
            }    
        );

        let result = try_join_all(requests).await;
        
        if result.is_ok() {
            let result = result.unwrap();
            return (
                result.iter()
                    .map(|item| &item.1)
                    .fold(None, |result, item| {
                        return match item {
                            Some(item) if result.is_none() || item.retrieve_timestamp < result.as_ref().unwrap().retrieve_timestamp => Some(item.clone()),
                            _ => result
                        };
                    }),
                result.into_iter()
                    .flat_map(|item| item.0)
                    .collect()
            );
        } else {
            return (None, Vec::new());
        }
    }
}

impl Reddit {
    async fn process_response(task: ParsingTask, response: Result<Response, reqwest::Error>, token: String) -> (Vec<ParsingTask>, Option<HttpAuthData>) {
        if let Ok(response) = response {
            let (response_timestamp, millis_to_refresh, requests_limit) = Reddit::parse_limits_from_header(&response);
            let mut new_parsing_tasks: Vec<ParsingTask> = Vec::new();
            if response.status() == StatusCode::OK {
                let response_body = response.json::<ThreadPage>().await;
                if response_body.is_ok() {
                    let thread = response_body.unwrap();
                    new_parsing_tasks.extend(Reddit::spawn_new_tasks(&task, &thread));
                    insert_with_replace(Self::get_entities(thread)).await;
                } 
            } else {
                update_tasks_with_status(vec![task._id.unwrap()], ParsingTaskStatus::New).await;
                if response.status() == StatusCode::FORBIDDEN {
                    error!("TO DO: re auth if http 403!");
                }
            }
            return (
                new_parsing_tasks,
                Some(HttpAuthData{ 
                    token: token, 
                    retrieve_timestamp: response_timestamp, 
                    millis_to_refresh, 
                    requests_limit 
                })
            );
        } else {
            return (Vec::new(), None);
        }

    }

    fn unfold_all(parsing_parameter: &RedditParsingParameters) -> Vec<ParsingTask> {
        let mut parsing_tasks: Vec<ParsingTask> = Vec::new();
        match parsing_parameter.reddit_task_type {
            RedditTaskType::All => {
                for task_type in RedditTaskType::iter() {
                    let mut all_parameter_unfold = parsing_parameter.clone();
                    match task_type {
                        RedditTaskType::All => continue,
                        RedditTaskType::Post => continue,
                        _ => all_parameter_unfold.reddit_task_type = task_type
                    }
                    parsing_tasks.push(Self::create_parsing_task(all_parameter_unfold));
                }    
            },
            _ => panic!("this branch should not be called")
        }
        return parsing_tasks;
    }

    fn create_parsing_task(parsing_parameter: RedditParsingParameters) -> ParsingTask {
        return ParsingTask {
            _id: None,
            execution_time: get_timestamp(),
            action_type: parsing_parameter.reddit_task_type.to_string(),
            parameters: ParsingTaskParameters::Reddit(parsing_parameter), 
            social_network: SocialNetworkEnum::Reddit,
            status: ParsingTaskStatus::New,
        };
    }

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
            .parse::<f32>().unwrap_or(0.0) as usize;
        return (timestamp, millis_to_refresh, requests_limit);
    }

    fn spawn_new_tasks(parsing_task: &ParsingTask, thread: &ThreadPage) -> Vec<ParsingTask> {
        let after = thread.posts.data.after.clone();
        return match after {
            Some(after) => vec![
                ParsingTask {
                    _id: None,
                    execution_time: get_timestamp(),
                    parameters: ParsingTaskParameters::Reddit(
                        RedditParsingParameters{ 
                            thread: parsing_task.parameters.as_ref_reddit().thread.clone(), 
                            reddit_task_type: parsing_task.parameters.as_ref_reddit().reddit_task_type, 
                            after: Some(after), 
                            id: None 
                        }),
                    action_type: parsing_task.parameters.as_ref_reddit().reddit_task_type.to_string(),
                    social_network: SocialNetworkEnum::Reddit,
                    status: ParsingTaskStatus::New
            }],
            None => vec![],
        }

    }

    fn get_entities(thread: ThreadPage) -> Vec<Entity> {
        let mut entities: Vec<Entity> = Vec::new();
        thread.posts.data.children.into_iter().for_each(|item| entities.push(item.data.into()));
        return entities;
    }
}
