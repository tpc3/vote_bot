
use magic_crypt::{new_magic_crypt, MagicCrypt128, MagicCryptTrait};
use once_cell::sync::Lazy;
use sled::{Db, Iter};

use crate::config::CONFIG;

pub fn codeblock(s: &String) -> String {
    format!("```{}```", &s)
}
pub fn link(title: &String, link: &String) -> String {
    format!("[{}]({})", &title, &link)
}

static MAGICCRYPT: Lazy<MagicCrypt128> = Lazy::new(|| magiccrypt_init());
static DB: Lazy<Db> = Lazy::new(|| db_init());

fn magiccrypt_init() -> MagicCrypt128 {
    new_magic_crypt!(&CONFIG.key)
}

pub fn encrypt_str_to_base64(input: &String) -> String {
    MAGICCRYPT.encrypt_str_to_base64(input)
}

pub fn decrypt_base64_to_string(base64: &String) -> String {
    MAGICCRYPT
        .decrypt_base64_to_string(&base64)
        .unwrap_or(String::new())
}

fn db_init() -> Db {
    sled::open(&CONFIG.db).unwrap()
}

pub fn db_insert(key: &String, value: &String) {
    DB.insert(key.as_bytes(), value.as_bytes()).unwrap();
}

pub fn db_get(key: &String) -> String {
    let result = DB.get(key.as_bytes()).unwrap();
    if result == None {
        return String::new();
    }
    String::from_utf8(result.unwrap().to_vec()).unwrap()
}

pub fn db_remove(key: &String) {
    DB.remove(key).unwrap();
}

pub fn db_iter() -> Iter {
    DB.iter()
}