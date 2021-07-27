use once_cell::sync::Lazy;
use serde_derive::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub token: String,
    pub id: u64,
    pub key: String,
    pub shards: u64,
    pub db: String,
    pub infos: Infos,
}

#[derive(Deserialize)]
pub struct Infos {
    pub name: String,
    pub prefix: String,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| init());

pub fn init() -> Config {
    let file = fs::read_to_string("./config.toml").expect("Conf file read error");
    toml::from_str(&file).unwrap()
}
