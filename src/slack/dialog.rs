extern crate serde_json;

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
    pub elements: Vec<element::Element>,
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
    pub submission: ::ip::RawEntry,
    pub callback_id: String,
    pub team: super::Team,
    pub user: super::User,
    pub channel: super::Channel,
    pub action_ts: String,
    pub token: String,
}

#[derive(Serialize)]
pub struct SubmissionError {
    pub name: String,
    pub error: String,
}

pub fn open(req: OpenRequest) -> super::super::error::Result<()> {
    let mut hm = ::std::collections::HashMap::new();
    hm.insert("token".to_owned(), req.token);
    hm.insert("dialog".to_owned(), serde_json::to_string(&req.dialog)?);
    hm.insert("trigger_id".to_owned(), req.trigger_id);
    let response: OpenResponse = super::request("dialog.open", &hm)?;

    if !response.ok {
        return Err(From::from("Bad Slack Response"));
    }

    Ok(())
}

pub mod element {
    extern crate serde_derive;

    #[derive(Serialize)]
    #[serde(tag = "type")]
    pub enum Element {
        #[serde(rename = "text")] Text(TextElement),
        #[serde(rename = "textarea")] TextArea(TextAreaElement),
        #[serde(rename = "select")] Select(SelectElement),
    }

    #[derive(Serialize)]
    pub struct TextElement {
        pub label: String,
        pub name: String,
        pub optional: Option<bool>,
        pub hint: Option<String>,
        pub subtype: Option<String>,
        pub value: Option<String>,
        pub placeholder: Option<String>,
    }

    #[derive(Serialize)]
    pub struct TextAreaElement {
        pub label: String,
        pub name: String,
        pub optional: Option<bool>,
        pub hint: Option<String>,
        pub subtype: Option<String>,
        pub value: Option<String>,
        pub placeholder: Option<String>,
    }

    #[derive(Serialize)]
    pub struct SelectElement {
        pub label: String,
        pub name: String,
        pub optional: Option<bool>,
        pub options: Vec<SelectOption>,
        pub value: Option<String>,
        pub placeholder: Option<String>,
    }

    #[derive(Serialize, Clone)]
    pub struct SelectOption {
        pub label: String,
        pub value: String,
    }
}
