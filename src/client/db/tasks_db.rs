use std::{collections::HashMap, hash::Hash};

use mongodb::{bson::{doc, self, oid::ObjectId}};
use serde::{Serialize, Deserialize};

use crate::{
    commons::{
        social_network::SocialNetworkEnum, 
        parsing_tasks::{
            ParsingTask, 
            ParsingTaskStatus
        }
    }, 
    utils::time::get_timestamp
};

use super::client::{DATABASE, DATABASE_COLLECTIONS, insert_if_not_empty, get_collection, GroupBoundaries};

use futures::{StreamExt};

#[derive(Clone, Copy)]
pub enum Limit {
    Limit(u64),
    NoLimit
}

pub async fn insert_tasks(tasks: &Vec<ParsingTask>) {
    insert_if_not_empty::<ParsingTask>(tasks, DATABASE::MANSA, DATABASE_COLLECTIONS::PARSING_TASKS).await;
}

pub async fn update_task_with_status(id: ObjectId, status: ParsingTaskStatus) {
    let match_query = doc! {
        "id" : {
            "$eq" : id
        }
    };
    let update_query = doc! {
        "$set": {
            "status": status.to_string()
        }
    };
    get_collection::<ParsingTask>()
        .await
        .update_one(match_query, update_query, None)
        .await
        .expect("unable to update tasks");
}

pub async fn update_tasks_with_status(ids: Vec<bson::oid::ObjectId>, status: ParsingTaskStatus) {
    let match_query = doc! {
        "_id" : {
            "$in" : ids
        }
    };
    let update_query = doc! {
        "$set": {
            "status": status.to_string()
        }
    };
    get_collection::<ParsingTask>()
        .await
        .update_many(match_query, update_query, None)
        .await
        .expect("unable to update tasks");
}

pub async fn get_tasks_sorted_by_exec_time(statuses: Vec<ParsingTaskStatus>, limit: Limit) -> Vec<ParsingTask> {
    let statuses: Vec<String> = statuses.into_iter().map(|item| item.to_string()).collect();
    let match_query = doc! {
        "$match": {
            "status": {
                "$in": statuses
            },
            "execution_time": {
                "$lt": get_timestamp() as i64
            }
        } 
    };
    let sort_query = doc! {
        "$sort": {
            "execution_time": 1
        }
    };
    let pipeline_fetch = match limit {
        Limit::Limit(number) => vec![match_query, sort_query, doc! {"$limit": number as i64}],
        Limit::NoLimit => vec![match_query, sort_query],
    }; 
    return  get_collection::<ParsingTask>()
        .await
        .aggregate(pipeline_fetch, None)
        .await
        .expect("unable to get parsing tasks")
        .with_type::<ParsingTask>()
        .map(|item| item.expect("unable unwrap parsing task from cursor stream"))
        .collect()
        .await;
}


#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq, Debug)]
pub struct GroupedTasks {
    pub _id: GroupBoundaries<SocialNetworkEnum>,
    pub tasks: Vec<ParsingTask>
}

impl GroupedTasks {
    pub fn to_hashmap(vec: Vec<GroupedTasks>) -> HashMap<SocialNetworkEnum, Vec<ParsingTask>> {
        let mut map: HashMap<SocialNetworkEnum, Vec<ParsingTask>> = HashMap::new();
        for item in vec.into_iter() {
            map.insert(item._id.min, item.tasks);
        }
        return map;
    }
}

pub async fn get_tasks_grouped_by_social_network() -> Vec<GroupedTasks> {
    let match_query = doc! {
        "$match": {
            "status": ParsingTaskStatus::New.to_string(),
            "execution_time": {
                "$lt": get_timestamp() as i64
            }
        } 
    };
    let sort_query = doc! {
        "$sort": {
            "execution_time": 1
        }
    };
    let bucket_query = doc! {
        "$bucketAuto": {
            "groupBy": "$social_network",
            "buckets": 100,
            "output" : {
                "tasks": { 
                    "$push": "$$ROOT"
                  }
            }
        }

    };
    let pipeline_fetch = vec![match_query, sort_query, bucket_query]; 
    return  get_collection::<ParsingTask>()
        .await
        .aggregate(pipeline_fetch, None)
        .await
        .expect("unable to get parsing tasks")    
        .with_type::<GroupedTasks>()
        .map(|item| item.expect("unable unwrap parsing task from cursor stream"))
        .collect()
        .await;
}

//needs special setup to work with transactions 
/*
pub async fn get_tasks_sorted_by_exec_time(limit: Limit) -> Vec<ParsingTask> {
    return TRANSACTION(
        async move |mut session|  {
            let r = get_tasks_sorted_by_exec_time_transactional(&mut session, limit).await;
            return (r, session);
        }
    ).await;
}

async fn get_tasks_sorted_by_exec_time_transactional(session: &mut ClientSession, limit: Limit) -> Vec<ParsingTask> {
    let match_query = doc! {
        "$match": {
            "status": ParsingTaskStatus::New.to_string(),
            "execution_time": {
                "$gt": get_timestamp() as i64
            }
        } 
    };
    let sort_query = doc! {
        "$sort": {
            "execution_time": 1
        }
    };
    let limit = doc! {
        "$limit": match limit {
            Limit::Limit(number) => number as i64,
            Limit::NoLimit => 0,
        }
    };
    let pipeline_fetch = vec![match_query.clone(), sort_query, limit];
    let result = get_collection::<ParsingTask>()
        .await
        .aggregate_with_session(pipeline_fetch, None, session)
        .await
        .expect("unable to get parsing tasks")
        .with_type::<ParsingTask>()
        .stream(session)
        .map(|item| item.expect("unable unwrap parsing task from cursor stream"))
        .collect()
        .await;

    let update_query = doc! {
        "$set": {
            "is_finished": ParsingTaskStatus::Processing.to_string()
        }
    };
    get_collection::<ParsingTask>()
        .await
        .update_many_with_session(match_query, update_query, None, session)
        .await
        .expect("unable to update fetched parsing tasks");
    
    return result;
} */
