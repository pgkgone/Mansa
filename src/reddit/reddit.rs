use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration};
use futures::FutureExt;
use log::{error, info};
use reqwest::{Response, StatusCode};
use crate::client::parser_v2::account_manager::account::{AccountSession, Account, AccountPtr, AccountDataPtr, ReqwestClientPtr};
use crate::client::parser_v2::statistics::STATISTICS;
use crate::client::settings::{SettingsPtr};
use crate::utils::time::get_timestamp;
use crate::client::db::tasks_db::{update_tasks_with_status, insert_tasks};
use crate::client::db::entities_db::{insert_with_replace};
use crate::commons::social_network::*;
use crate::commons::parsing_tasks::*;
use crate::commons::entity::Entity;
use super::{
    reddit_parsing_task::RedditParsingTask, 
    data_types::{
        reddit_auth::AuthResponse, 
        reddit_pages::{
            ThreadPage, 
            CommentPage
        }, 
        reddit_response_body::ResponseBody
    }
};

pub struct Reddit {
    pub auth_url: String,
    pub enable_comments_parsing: bool
}

impl Default for Reddit {
    fn default() -> Self {
        Self { 
            auth_url: String::from("https://www.reddit.com/api/v1/access_token") ,
            enable_comments_parsing: true
        }
    }
}

unsafe impl Sync for Reddit {}

#[async_trait]
impl SocialNetwork for Reddit {
    
    async fn auth(
        &self, 
        account_data: AccountDataPtr, 
        client: ReqwestClientPtr
    ) -> Result<AccountSession, Box<dyn Error + Send + Sync>> {
        let response = client
            .post(self.auth_url.clone())
            .basic_auth(
                account_data.public_key.as_ref().ok_or("no public key")?, 
                account_data.private_key.as_ref()
            )
            .body(format!(
                "grant_type=password&username={}&password={}", 
                account_data.login.as_ref().ok_or("no login")?,
                account_data.password.as_ref().ok_or("no password")?
            ))
            .send()
            .await?;
        let (_, millis_to_refresh, requests_limit) = Reddit::parse_limits_from_header(&response);
        let auth_response_json = response.json::<AuthResponse>()
            .await?;
        return Ok(AccountSession{ 
            token: auth_response_json.access_token, 
            retrieve_timestamp: get_timestamp(), 
            millis_to_refresh, 
            requests_limit: requests_limit 
        });
    }

    async fn parse(&self, parsing_task: ParsingTask, account: AccountPtr) {
        let reqwest_client = account.reqwest_client.clone().expect("Unauthorized account in Reddit parser");
        let mut token = String::from("");
        {
            let rg_session = account.session.read().await;
            token = rg_session.as_ref().expect("Unauthorized account in Reddit parser").token.clone();
        }
        reqwest_client
            .get(parsing_task.parameters.as_ref_reddit().to_url())
            .bearer_auth(token)
            .send()
            .then( 
                move |response| Reddit::process_response(parsing_task, account.clone(), response)
            ).await;
    }

    fn apply_settings(&mut self, settings: SettingsPtr) {
        self.enable_comments_parsing = settings
            .social_network_settings.get(&SocialNetworkEnum::Reddit).unwrap()
            .additional_properties.get("enable_comments_parsing").unwrap()
            .as_bool().unwrap();
    }

    fn prepare_parsing_tasks(&self, settings: SettingsPtr) ->  Result<Vec<ParsingTask>, Box<dyn Error>> {
        let tasks = &settings.social_network_settings
            .get(&SocialNetworkEnum::Reddit).unwrap().parsing_tasks;

        let mut parsing_tasks: Vec<RedditParsingTask> = Vec::new();

        for task in tasks {
            parsing_tasks.extend(RedditParsingTask::get_all(
                &task.get("thread").unwrap().as_str().unwrap().to_string()
            ));
        }

        return Ok(
            parsing_tasks
                .into_iter()
                .map(|item| ParsingTask{ 
                    _id: None, 
                    execution_time: get_timestamp(), 
                    action_type: item.to_string(), 
                    parameters: ParsingTaskParameters::Reddit(item), 
                    social_network: SocialNetworkEnum::Reddit, 
                    status: ParsingTaskStatus::New }
                )
                .collect()
        );
    }

    fn prepare_accounts(&self, settings: SettingsPtr) -> Result<Vec<Account>, Box<dyn Error>> {
        let settings_accounts = settings.social_network_settings.get(&SocialNetworkEnum::Reddit)
            .as_ref().unwrap()
            .accounts.iter()
            .map(|settings_account| Account::new(Arc::new(settings_account.clone()), None))
            .collect();
        return Ok(settings_accounts);
    }
}


impl Reddit {
    async fn process_response(task: ParsingTask, account: AccountPtr, response: Result<Response, reqwest::Error>) {
        match response {
            Ok(response) => {
                let (response_timestamp, millis_to_refresh, requests_limit) = Reddit::parse_limits_from_header(&response);
                if response.status() == StatusCode::OK {
                    let response_url = response.url().to_string().clone();
                    info!("Recived status 200. Url: {}", response_url);
                    let response_body = match task.parameters.as_ref_reddit() {
                        RedditParsingTask::Post { .. } => {
                            info!("Successfully recived post page");
                            ResponseBody::Comments(
                                response.json::<CommentPage>().await.inspect_err(|e| error!("unable to parse Post {} {}", response_url, e))
                            )
                        },
                        _ =>  {
                            info!("Successfully recived thread page");
                            ResponseBody::Thread(
                                response.json::<ThreadPage>().await.inspect_err(|_| error!("unable to parse Thread {}", response_url))
                            )
                        }
                    };
                    if response_body.is_ok() {
                        insert_tasks(&Reddit::spawn_new_tasks(&task, &response_body)).await;
                        insert_with_replace(Self::get_entities(response_body)).await;
                        update_tasks_with_status(vec![task._id.unwrap()], ParsingTaskStatus::Processed).await;
                    }
                    let mut wg_session = account.session.write().await;
                    let session = wg_session.as_mut().unwrap();
                    session.retrieve_timestamp = response_timestamp;
                    session.millis_to_refresh = millis_to_refresh;
                    session.requests_limit = requests_limit;
                } else {
                    STATISTICS.increase_failed_parsing_tasks();
                    info!("Recived status: {}", response.status());
                    update_tasks_with_status(vec![task._id.unwrap()], ParsingTaskStatus::New).await;
                    if response.status() == StatusCode::FORBIDDEN {
                        STATISTICS.increase_access_failed_parsing_tasks();
                        account.auth().await;
                    }
                }
            },
            Err(err) => {
                info!("error {}", err);
            },
        }
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

    fn spawn_new_tasks(parsing_task: &ParsingTask, response_body: &ResponseBody) -> Vec<ParsingTask> {
        return match response_body {
            ResponseBody::Thread(thread) => {
                let thread = thread.as_ref().unwrap();
                let after = thread.posts.data.after.clone();
                let thread_task = match after {
                    Some(after) => vec![
                        ParsingTask {
                            _id: None,
                            execution_time: get_timestamp(),
                            parameters: ParsingTaskParameters::Reddit(
                                parsing_task.parameters.as_ref_reddit().with_parameter(Some(after))
                            ),
                            action_type: parsing_task.parameters.as_ref_reddit().to_string(),
                            social_network: SocialNetworkEnum::Reddit,
                            status: ParsingTaskStatus::New
                    }],
                    None => vec![],
                };
                let mut post_tasks = thread.posts.data.children.iter()
                    .map(|item| ParsingTask {
                        _id: None,
                        execution_time: get_timestamp(),// + Duration::hours(6).num_milliseconds() as u64,
                        parameters: ParsingTaskParameters::Reddit(
                            RedditParsingTask::Post {
                                thread: parsing_task.parameters.as_ref_reddit().get_thread(), 
                                id: Some(item.data.id.clone()),
                                update_number: 5
                            }
                        ),
                        action_type: parsing_task.parameters.as_ref_reddit().to_string(),
                        social_network: SocialNetworkEnum::Reddit,
                        status: ParsingTaskStatus::New
                    }).collect::<Vec<_>>();
                post_tasks.extend(thread_task);
                return post_tasks;
            },
            ResponseBody::Comments(comments) => {
                match &parsing_task.parameters.as_ref_reddit() {
                    RedditParsingTask::Post { thread, id, update_number } => {
                        let mut new_task = vec![];
                        if *update_number <= 2 {
                            new_task = vec![
                                ParsingTask { 
                                    _id: None, 
                                    execution_time: get_timestamp() + Duration::hours(1).num_milliseconds() as u64, 
                                    parameters: ParsingTaskParameters::Reddit(
                                        RedditParsingTask::Post { 
                                            thread: thread.clone(), 
                                            id: Some(id.as_ref().unwrap().clone()), 
                                            update_number: update_number + 1 
                                        }
                                    ), 
                                    action_type: parsing_task.parameters.as_ref_reddit().to_string(),
                                    social_network: SocialNetworkEnum::Reddit, 
                                    status: ParsingTaskStatus::New 
                                }
                            ]
                        }
                        return new_task
                    },
                    _ => {vec![]}
                }
            },
        };
    }

    fn get_entities(thread: ResponseBody) -> Vec<Entity> {
        let mut entities: Vec<Entity> = Vec::new();
        match thread {
            ResponseBody::Thread(thread) => {
                thread.unwrap().posts.data.children.into_iter().for_each(|item| entities.push(item.data.into()));
            },
            ResponseBody::Comments(comments) => {
                let mut comments = comments.unwrap();
                comments.comments.data.children.into_iter().for_each(|item| entities.push(item.data.into()));
            },
        }
        return entities;
    }
}


