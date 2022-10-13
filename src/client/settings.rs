use std::{collections::HashMap, fs::{self, File}, io::BufReader, sync::Arc};

use derivative::Derivative;
use log::info;
use serde::{Serialize, Deserialize};
use serde_json::Value;
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

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(PartialEq, Debug, Hash, Eq)]
pub struct Settings {
    pub general_settings: GeneralSettings,
    #[derivative(Hash="ignore", Debug="ignore")]
    pub social_network_settings: HashMap<SocialNetworkEnum, SocialNetworkSettings>
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug)]
pub struct GeneralSettings {
    pub proxies: Vec<Proxy>,
    pub disable_proxy: bool
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(PartialEq, Hash, Eq)]
pub struct SocialNetworkSettings {
    pub social_network: SocialNetworkEnum,
    pub accounts: Vec<Account>,
    #[derivative(Hash="ignore")]
    pub parsing_tasks: Vec<HashMap<String, Value>>,
    #[derivative(Hash="ignore")]
    pub additional_properties: HashMap<String, Value>
}


enum SettingsPathType {
    GeneralSettingsPath(String),
    SocialNetworkSettingsPath(SocialNetworkEnum, String),
    OtherPath()
}



impl SettingsPathType {

    fn test(path: String) -> SettingsPathType {
        for social_net in SocialNetworkEnum::iter() {
            if path.contains(&social_net.to_string()) {
                return SettingsPathType::SocialNetworkSettingsPath(social_net, path);
            }
        }
        if path.contains("general_settings.json") {
            return SettingsPathType::GeneralSettingsPath(path);
        } else {
            return SettingsPathType::OtherPath();
        }
    }

}

pub fn get_settings() -> SettingsPtr {

    info!("parsing settings files");
    let paths = fs::read_dir("./settings").expect("not found settings dir");

    let mut general_settings: Option<GeneralSettings> = None;

    let mut social_network_settings: HashMap<SocialNetworkEnum, SocialNetworkSettings> = HashMap::new();

    for path in paths {

        match SettingsPathType::test(path.unwrap().path().to_str().unwrap().to_string()) {

            SettingsPathType::GeneralSettingsPath(general_settings_path) => {
                let f = File::open(general_settings_path).expect("unable to read general_settings file");
                let mut reader = BufReader::new(f);
                general_settings = Some(serde_json::from_reader(reader).expect("unable to parse general_settings file"));
            },

            SettingsPathType::SocialNetworkSettingsPath(social_net, social_network_settings_folder) => {
                let f = File::open(social_network_settings_folder + "/settings.json").expect(&format!("unable to open settings file of"));
                let mut reader = BufReader::new(f);
                social_network_settings.insert(
                    social_net, 
                    serde_json::from_reader(reader).expect("unable to parse reddit social net settings file")
                );
            },

            SettingsPathType::OtherPath() => {}
        }

    }
    return Arc::new(Settings {
        general_settings: general_settings.expect("general settings should not be empty"),
        social_network_settings,
    })
}

