extern crate rocket_contrib;
extern crate serde_derive;
extern crate serde_json;

use super::ip;
use super::slack::slash_command;
use super::slack::dialog::{open, Dialog, OpenRequest, Submission};
use super::slack::dialog::element::*;
use super::settings;
use std::error::Error;
use serde_json::Value as Json;

pub enum Command {
    Add,
    Get,
    Edit,
    Issue,
}

pub struct App {
    settings: settings::Settings,
}

impl App {
    pub fn new(settings: settings::Settings) -> App {
        App { settings }
    }

    pub fn handle_command(&self, command: Command, data: slash_command::Request) -> Json {
        use self::Command::*;

        if !self.validate(&data.token) {
            return json!({"text": "Error: Validation error".to_owned()});
        }

        match match command {
            Add => self.add_command(data),
            Get => self.get_command(data),
            _ => Ok(json!({"text": "Nope".to_owned()})),
        } {
            Ok(s) => s,
            Err(e) => json!({
                "text": format!("Error: {}", e)
            }),
        }
    }

    pub fn handle_submission(&self, submission: Submission) -> String {
        "".to_owned()
    }

    fn validate(&self, token: &str) -> bool {
        token == self.settings.verification_token
    }

    fn add_command(&self, data: slash_command::Request) -> Result<Json, Box<Error>> {
        self.show_add_dialog(data)?;
        Ok(json!({"text": "Ok".to_owned()}))
    }

    fn show_add_dialog(&self, data: slash_command::Request) -> Result<(), Box<Error>> {
        let mut dialog = Dialog::new("add".to_owned(), "IP 추가".to_owned());

        dialog.elements.push(generate_ip_text(None).into_json()?);
        dialog
            .elements
            .push(generate_domain_text(None).into_json()?);
        dialog
            .elements
            .push(generate_using_select("false".to_owned()).into_json()?);
        dialog
            .elements
            .push(generate_open_ports_text(None).into_json()?);
        dialog
            .elements
            .push(generate_description_textarea(None).into_json()?);

        let request = OpenRequest {
            token: self.settings.api_token.clone(),
            dialog,
            trigger_id: data.trigger_id,
        };
        open(request)?;

        Ok(())
    }

    pub fn get_command(&self, data: slash_command::Request) -> Result<Json, Box<Error>> {
        use super::ip::get;
        if data.text.is_empty() {
            return Err(Box::new(::std::io::Error::new(
                ::std::io::ErrorKind::Other,
                "Invalid argument",
            )));
        }
        let entry = get(&data.text);
        match entry {
            Ok(e) => Ok(serde_json::to_value(show_get_info(&e))?),
            Err(_) => Err(Box::new(::std::io::Error::new(
                ::std::io::ErrorKind::Other,
                "IP not found",
            ))),
        }
    }

    pub fn edit_command(&self, data: slash_command::Request) -> Result<String, Box<Error>> {
        Ok("edit".to_owned())
    }

    pub fn issue_command(&self, data: slash_command::Request) -> Result<String, Box<Error>> {
        Ok("issue".to_owned())
    }
}

fn show_get_info(entry: &ip::Entry) -> ::slack::AttachedMessage {
    use slack::*;
    let mut m = AttachedMessage {
        attachments: vec![],
    };
    let mut a = Attachment { fields: vec![] };

    a.fields.push(AttachmentFields {
        title: "IP".to_owned(),
        value: entry.ip.clone(),
    });
    if let Some(ref domain) = entry.domain {
        a.fields.push(AttachmentFields {
            title: "도메인".to_owned(),
            value: domain.clone(),
        });
    }
    a.fields.push(AttachmentFields {
        title: "사용 여부".to_owned(),
        value: if entry.using {
            "사용중"
        } else {
            "미사용"
        }.to_owned(),
    });

    if !entry.open_ports.is_empty() {
        let mut s = String::new();
        for p in &entry.open_ports {
            s.push_str(&format!("{}, ", p));
        }
        s.pop();
        s.pop();
        s.push('\n');
        a.fields.push(AttachmentFields {
            title: "개방 포트".to_owned(),
            value: s,
        });
    }
    if let Some(ref description) = entry.description {
        a.fields.push(AttachmentFields {
            title: "설명".to_owned(),
            value: description.clone(),
        });
    }
    m.attachments.push(a);
    m
}

fn generate_ip_text(value: Option<String>) -> super::slack::dialog::element::Text {
    super::slack::dialog::element::Text {
        label: "IP".to_owned(),
        name: "ip".to_owned(),
        optional: None,
        hint: None,
        subtype: None,
        value,
        placeholder: None,
    }
}

fn generate_domain_text(value: Option<String>) -> super::slack::dialog::element::Text {
    super::slack::dialog::element::Text {
        label: "도메인".to_owned(),
        name: "domain".to_owned(),
        optional: Some(true),
        hint: None,
        subtype: None,
        value,
        placeholder: Some("Optional".to_owned()),
    }
}

fn generate_using_select(value: String) -> super::slack::dialog::element::Select {
    use super::slack::dialog::element::{Select, SelectOption};

    Select {
        label: "사용 여부".to_owned(),
        name: "using".to_owned(),
        optional: None,
        options: vec![
            SelectOption {
                label: "사용".to_owned(),
                value: "true".to_owned(),
            },
            SelectOption {
                label: "미사용".to_owned(),
                value: "false".to_owned(),
            },
        ],
        value,
        placeholder: None,
    }
}

fn generate_open_ports_text(value: Option<String>) -> super::slack::dialog::element::Text {
    super::slack::dialog::element::Text {
        label: "개방된 포트".to_owned(),
        name: "open_ports".to_owned(),
        optional: Some(true),
        hint: Some("쉼표로 구분".to_owned()),
        subtype: None,
        value,
        placeholder: Some("ex) 22, 80".to_owned()),
    }
}

fn generate_description_textarea(value: Option<String>) -> super::slack::dialog::element::TextArea {
    super::slack::dialog::element::TextArea {
        label: "설명".to_owned(),
        name: "description".to_owned(),
        optional: Some(true),
        hint: None,
        subtype: None,
        value,
        placeholder: Some("Optional".to_owned()),
    }
}
