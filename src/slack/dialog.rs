extern crate serde_json;

use std::collections::HashMap;
use super::{request, Channel, Team, User};
use std::error::Error;

#[derive(Serialize)]
pub struct OpenRequest {
    pub token: String,
    pub dialog: Dialog,
    pub trigger_id: String,
}

#[derive(Deserialize)]
pub struct OpenResponse {
    pub ok: bool,
}

#[derive(Serialize)]
pub struct Dialog {
    pub callback_id: String,
    pub title: String,
    pub elements: Vec<serde_json::Value>,
}

impl Dialog {
    pub fn new(callback_id: String, title: String) -> Dialog {
        Dialog {
            callback_id,
            title,
            elements: vec![],
        }
    }
}

#[derive(FromForm)]
pub struct SubmissionResponse {
    pub payload: String,
}

#[derive(Deserialize)]
pub struct Submission {
    #[serde(rename = "type")] pub submission_type: String,
    pub submission: Vec<HashMap<String, String>>,
    pub callback_id: String,
    pub team: Team,
    pub user: User,
    pub channel: Channel,
    pub action_ts: String,
    pub token: String,
}

#[derive(Serialize)]
pub struct SubmissionError {
    pub name: String,
    pub error: String,
}

pub fn open(req: OpenRequest) -> Result<(), Box<Error>> {
    let mut hm = HashMap::new();
    hm.insert("token".to_owned(), req.token);
    println!("{}", serde_json::to_string_pretty(&req.dialog)?);
    hm.insert("dialog".to_owned(), serde_json::to_string(&req.dialog)?);
    hm.insert("trigger_id".to_owned(), req.trigger_id);
    let response: OpenResponse = request("dialog.open", hm)?;

    if !response.ok {
        return Err(Box::new(::std::io::Error::new(
            ::std::io::ErrorKind::Other,
            "Bad Slack Response",
        )));
    }

    Ok(())
}

pub mod element {
    extern crate serde_json;

    use self::serde_json::Value;
    use self::serde_json::Map;
    use self::serde_json::Result;

    pub trait Element {
        fn into_json(self) -> Result<Value>;
    }

    pub struct Text {
        pub label: String,
        pub name: String,
        pub optional: Option<bool>,
        pub hint: Option<String>,
        pub subtype: Option<String>,
        pub value: Option<String>,
        pub placeholder: Option<String>,
    }

    impl Element for Text {
        fn into_json(self) -> Result<Value> {
            let mut map = Map::new();
            map.insert("label".to_owned(), serde_json::to_value(self.label)?);
            map.insert("name".to_owned(), serde_json::to_value(self.name)?);
            map.insert("type".to_owned(), serde_json::to_value("text")?);
            if let Some(optional) = self.optional {
                map.insert(
                    "optional".to_owned(),
                    serde_json::to_value(if optional { "true" } else { "false" })?,
                );
            }
            if let Some(ref hint) = self.hint {
                map.insert("hint".to_owned(), serde_json::to_value(hint)?);
            }
            if let Some(ref subtype) = self.subtype {
                map.insert("subtype".to_owned(), serde_json::to_value(subtype)?);
            }
            if let Some(ref value) = self.value {
                map.insert("value".to_owned(), serde_json::to_value(value)?);
            }
            if let Some(ref placeholder) = self.placeholder {
                map.insert("placeholder".to_owned(), serde_json::to_value(placeholder)?);
            }
            Ok(Value::Object(map))
        }
    }

    pub struct TextArea {
        pub label: String,
        pub name: String,
        pub optional: Option<bool>,
        pub hint: Option<String>,
        pub subtype: Option<String>,
        pub value: Option<String>,
        pub placeholder: Option<String>,
    }

    impl Element for TextArea {
        fn into_json(self) -> Result<Value> {
            let mut map = Map::new();
            map.insert("label".to_owned(), serde_json::to_value(self.label)?);
            map.insert("name".to_owned(), serde_json::to_value(self.name)?);
            map.insert("type".to_owned(), serde_json::to_value("textarea")?);
            if let Some(optional) = self.optional {
                map.insert(
                    "optional".to_owned(),
                    serde_json::to_value(if optional { "true" } else { "false" })?,
                );
            }
            if let Some(ref hint) = self.hint {
                map.insert("hint".to_owned(), serde_json::to_value(hint)?);
            }
            if let Some(ref subtype) = self.subtype {
                map.insert("subtype".to_owned(), serde_json::to_value(subtype)?);
            }
            if let Some(ref value) = self.value {
                map.insert("value".to_owned(), serde_json::to_value(value)?);
            }
            if let Some(ref placeholder) = self.placeholder {
                map.insert("placeholder".to_owned(), serde_json::to_value(placeholder)?);
            }
            Ok(Value::Object(map))
        }
    }

    pub struct Select {
        pub label: String,
        pub name: String,
        pub optional: Option<bool>,
        pub options: Vec<SelectOption>,
        pub value: String,
        pub placeholder: Option<String>,
    }

    #[derive(Serialize, Clone)]
    pub struct SelectOption {
        pub label: String,
        pub value: String,
    }

    impl Element for Select {
        fn into_json(self) -> Result<Value> {
            let mut map = Map::new();
            map.insert("label".to_owned(), serde_json::to_value(self.label)?);
            map.insert("name".to_owned(), serde_json::to_value(self.name)?);
            map.insert("type".to_owned(), serde_json::to_value("select")?);
            if let Some(optional) = self.optional {
                map.insert(
                    "optional".to_owned(),
                    serde_json::to_value(if optional { "true" } else { "false" })?,
                );
            }
            map.insert("options".to_owned(), serde_json::to_value(self.options)?);
            map.insert("value".to_owned(), serde_json::to_value(self.value)?);
            if let Some(placeholder) = self.placeholder {
                map.insert("placeholder".to_owned(), serde_json::to_value(placeholder)?);
            }
            Ok(Value::Object(map))
        }
    }

}
