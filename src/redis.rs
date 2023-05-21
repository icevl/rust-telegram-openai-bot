use redis::{Client, Commands};
use std::error::Error;

lazy_static::lazy_static! {
    static ref REDIS_CLIENT: redis::Client = {
        let client = Client::open("redis://127.0.0.1/").unwrap();
        client
    };
}

pub fn set_key<T: redis::ToRedisArgs>(key: &str, value: T) {
    let mut connection = REDIS_CLIENT.get_connection().unwrap();
    let _: () = connection.set(key, value).unwrap();
}

pub fn get_key(key: &String) -> Result<String, Box<dyn Error>> {
    let mut connection = REDIS_CLIENT.get_connection().unwrap();
    let result: Option<String> = connection.get(key)?;

    match result {
        Some(value) => Ok(value),
        None => Err("Key not found".into()),
    }
}
