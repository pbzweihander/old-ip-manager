extern crate reqwest;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

pub mod slash_command;
pub mod dialog;

use super::error::Result;

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

#[derive(Serialize)]
pub struct AttachedMessage {
    pub attachments: Vec<Attachment>,
}

#[derive(Serialize)]
pub struct Attachment {
    pub title: String,
    pub fields: Vec<AttachmentFields>,
}

#[derive(Serialize)]
pub struct AttachmentFields {
    pub title: String,
    pub value: String,
}

fn encode_url(url: &str) -> String {
    url.to_owned()
        .replace(" ", "%20")
        .replace("<", "%3C")
        .replace(">", "%3E")
        .replace("#", "%23")
}

pub fn request<R: serde::de::DeserializeOwned>(
    api: &str,
    argument: &::std::collections::HashMap<String, String>,
) -> Result<R> {
    let mut uri = String::from("https://slack.com/api/");
    uri.push_str(api);
    uri.push('?');
    for (key, val) in argument {
        uri.push_str(key);
        uri.push('=');
        uri.push_str(&encode_url(val));
        uri.push('&');
    }
    uri.pop();

    let resp = reqwest::get(&uri)?;

    let parsed: R = serde_json::from_reader(resp)?;

    Ok(parsed)
}
