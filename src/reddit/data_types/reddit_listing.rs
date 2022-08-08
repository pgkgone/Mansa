use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Listing<T> {
    pub kind: Option<String>, 
    pub data: Data<T>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Data<T> {
    pub after: Option<String> ,
    pub children: Vec<Children<T>>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Children<T> {
    pub kind: Option<String>, 
    pub data: T
}