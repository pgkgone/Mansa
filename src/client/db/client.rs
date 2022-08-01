use std::{sync::Arc, borrow::Borrow};

use async_once::AsyncOnce;
use lazy_static::lazy_static;
use log::{error, info};
use mongodb::{Client, options::ClientOptions, error::ErrorKind};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, Display};

use crate::{generic::entity::Entity, client::managers::task_manager::ParsingTask};


#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq, EnumIter, Display)]
pub enum DATABASE {
    MANSA
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq, EnumIter, Display)]
pub enum DATABASE_COLLECTIONS {
    ENTITIES,
    PARSING_TASKS
}

pub async fn create_mongo_client(login: &str, password: &str, port: u64) -> Client {
    let mut options = ClientOptions::parse(format!("mongodb://{}:{}@localhost:{}", login, password, port))
        .await
        .expect("unable to connect to mongo databse");
    info!("successfully set options");
    return Client::with_options(options)
        .inspect(|v| info!("successfully connects to db"))
        .expect("unable to connect to mongo databse");
}

lazy_static! {
    pub static ref MONGO_CLIENT: AsyncOnce<Arc<Client>> = AsyncOnce::new( async {
        return Arc::new(create_mongo_client("root", "example", 27017).await);
    });
}

pub async fn insert_if_not_empty<T>(collection: impl IntoIterator<Item = impl Borrow<T>>, database: DATABASE, database_collection: DATABASE_COLLECTIONS) 
where
    T: Serialize
{
    let r = MONGO_CLIENT
        .get()
        .await
        .database(&database.to_string())
        .collection::<T>(&database_collection.to_string())
        .insert_many(collection, None)
        .await;

    if r.is_err() {
        match *r.unwrap_err().kind {
            ErrorKind::InvalidArgument { message , .. } => info!("try to insert empty collection in db: {}-{}", database, database_collection),
            err => error!("unable to insert entities in db: {}-{}; {}", database, database_collection, err),
        }
    } else {
        info!("successfully inserted entities in db: {}-{}", database, database_collection)
    }
}
