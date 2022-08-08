use std::{collections::HashMap, fs::{self, File}, io::BufReader, sync::Arc};

use log::info;
use serde::{Serialize, Deserialize};
use strum::IntoEnumIterator;

use crate::{generic::{social_network::SocialNetworkEnum, parsing_tasks::{ParsingTaskParameters}}};

pub type SettingsPtr = Arc<Settings>;

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug, Clone)]
pub struct Account {
    pub login: Option<String>,
    pub password: Option<String>, 
    pub public_key: Option<String>, 
    pub private_key: Option<String>,
    pub social_network: SocialNetworkEnum
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug, Clone)]
pub struct Proxy {
    pub host: String, 
    pub login: Option<String>,
    pub password: Option<String>
}

impl From<&Proxy> for reqwest::Proxy {
    fn from(proxy: &Proxy) -> Self {

        if proxy.login.is_some() && proxy.password.is_some() {
            return reqwest::Proxy::https(proxy.host.clone())
                .unwrap()
                .basic_auth(
                    proxy.login.as_ref().unwrap(),
                    proxy.password.as_ref().unwrap()
                );
        } else {
            return reqwest::Proxy::https(proxy.host.clone())
            .unwrap();
        }  
           
    }
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug)]
pub struct SocialNetworkSettings {
    pub social_network: SocialNetworkEnum,
    pub accounts: Vec<Account>,
    pub parsing_tasks: Vec<ParsingTaskParameters>
    
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug)]
pub struct GeneralSettings {
    pub proxies: Vec<Proxy>,
    pub disable_proxy: bool
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug)]
pub struct Settings {
    pub general_settings: GeneralSettings,
    pub social_network_settings: Vec<SocialNetworkSettings>
}

enum SettingsPathType {
    GeneralSettingsPath(String),
    SocialNetworkSettingsPath(String),
    OtherPath()
}



impl SettingsPathType {

    fn test(path: String) -> SettingsPathType {
        if path.contains("general_settings.json") {
            return SettingsPathType::GeneralSettingsPath(path);
        } else if SocialNetworkEnum::iter().any(|enumName| path.contains(&enumName.to_string())){
            return SettingsPathType::SocialNetworkSettingsPath(path);
        } else {
            return SettingsPathType::OtherPath();
        }
    }

}

pub fn get_settings() -> SettingsPtr {

    info!("parsing settings files");
    let paths = fs::read_dir("./settings").expect("not found settings dir");

    let mut general_settings: Option<GeneralSettings> = None;

    let mut social_network_settings: Vec<SocialNetworkSettings> = Vec::with_capacity(SocialNetworkEnum::iter().count());

    for path in paths {

        match SettingsPathType::test(path.unwrap().path().to_str().unwrap().to_string()) {

            SettingsPathType::GeneralSettingsPath(general_settings_path) => {
                let f = File::open(general_settings_path).expect("unable to read general_settings file");
                let mut reader = BufReader::new(f);
                general_settings = Some(serde_json::from_reader(reader).expect("unable to parse general_settings file"));
            },

            SettingsPathType::SocialNetworkSettingsPath(social_network_settings_folder) => {
                let f = File::open(social_network_settings_folder + "/settings.json").expect(&format!("unable to open settings file of"));
                let mut reader = BufReader::new(f);
                social_network_settings.push(serde_json::from_reader(reader).expect("unable to parse reddit social net settings file"));
            },

            SettingsPathType::OtherPath() => {}
        }

    }
    return Arc::new(Settings {
        general_settings: general_settings.expect("general settings should not be empty"),
        social_network_settings,
    })
}

