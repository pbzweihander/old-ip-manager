extern crate reqwest;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashMap;
use std::error::Error;

#[derive(Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub domain: String,
}

fn encode_url(url: String) -> String {
    url.replace(" ", "%20")
        .replace("<", "%3C")
        .replace(">", "%3E")
        .replace("#", "%23")
}

pub fn request<R: serde::de::DeserializeOwned>(
    api: &str,
    argument: HashMap<String, String>,
) -> Result<R, Box<Error>> {
    let mut uri = String::from("https://slack.com/api/");
    uri.push_str(api);
    uri.push('?');
    for (key, val) in &argument {
        uri.push_str(key);
        uri.push('=');
        uri.push_str(&encode_url(val.to_owned()));
        uri.push('&');
    }
    uri.pop();

    let resp = reqwest::get(&uri)?;

    let parsed: R = serde_json::from_reader(resp)?;

    Ok(parsed)
}

pub mod slash_command;
pub mod dialog;
