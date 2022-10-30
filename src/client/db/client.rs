use std::{sync::Arc, borrow::Borrow, cell::RefCell, rc::Rc};

use async_once::AsyncOnce;
use error::ErrorKind;
use futures::Future;
use lazy_static::lazy_static;
use log::{error, info};
use mongodb::{ClientSession, Collection};
use mongodb::error::{UNKNOWN_TRANSACTION_COMMIT_RESULT, TRANSIENT_TRANSACTION_ERROR, self};
use mongodb::options::{ClientOptions, TransactionOptions, ReadConcern, WriteConcern, Acknowledgment};
use mongodb::Client;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, Display};

use crate::commons::entity::Entity;


#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq, EnumIter, Display)]
pub enum DATABASE {
    MANSA
}

pub type ClientSessionPtr = Rc<RefCell<ClientSession>>;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq, EnumIter, Display)]
pub enum DATABASE_COLLECTIONS {
    ENTITIES,
    PARSING_TASKS
}

pub trait DBCollection {
    fn get_collection() -> String; 
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

    pub static ref ENTITY_COLLECTION: AsyncOnce<Arc<Collection<Entity>>> = AsyncOnce::new( async {
        return Arc::new(get_collection::<Entity>().await);
    }); 
}

pub async fn TRANSACTION<R, F, Fut>(func: F ) -> R
where
F:  Fn(ClientSession) -> Fut,
Fut: Future<Output = (R, ClientSession)>
{
    let mut session: ClientSession = MONGO_CLIENT
        .get().await
        .start_session(None).await.expect("unable to start session");
    let options = TransactionOptions::builder()
        .read_concern(ReadConcern::majority())
        .write_concern(WriteConcern::builder().w(Acknowledgment::Majority).build())
        .build();
    session.start_transaction(options)
        .await
        .expect("unable to start transaction");
    loop {
        let (func_result, session_r) = func(session).await;
        session = session_r;
        let mut commit_result = Ok(());
        loop {
            commit_result = session.commit_transaction().await;
            if let Err(ref error) = commit_result {
                if error.contains_label(UNKNOWN_TRANSACTION_COMMIT_RESULT) {
                    continue;
                }
            }
            break;
        }
        if let Ok(_) = commit_result {
            return func_result;
        }        
    }
}


pub async fn get_collection<T: DBCollection>() -> Collection<T> {
    return MONGO_CLIENT
        .get()
        .await
        .database(&DATABASE::MANSA.to_string())
        .collection::<T>(&T::get_collection());
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

#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq, Debug)]
pub struct GroupBoundaries<T> {
    pub min: T,
    pub max: T
}