#![feature(plugin, custom_derive, decl_macro)]
#![plugin(rocket_codegen)]
#[macro_use]
extern crate lazy_static;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

pub mod ip;
pub mod slack;
pub mod settings {
    extern crate config;

    #[derive(Deserialize)]
    pub struct Settings {
        pub verification_token: String,
        pub api_token: String,
        pub data_path: String,
    }

    impl Settings {
        pub fn new() -> Result<Settings, config::ConfigError> {
            use std::env::args;
            use std::path::Path;
            use self::config::{Config, File};
            let mut settings = Config::new();
            settings.merge(if args().len() >= 2 {
                File::from(Path::new(&args().nth(1).unwrap()))
            } else {
                File::with_name("settings")
            })?;
            settings.try_into::<Settings>()
        }
    }
}

use std::sync::RwLock;

lazy_static! {
    static ref SETTINGS: RwLock<settings::Settings> = RwLock::new(match settings::Settings::new() {
        Ok(s) => s,
        Err(e) => panic!("Settings file parse error!, {}", e)
    });
}

pub fn validate_data_path() {
    use std::fs::read_dir;
    if read_dir(&SETTINGS.read().unwrap().data_path).is_err() {
        panic!("Invalid data folder. Check settings file!");
    }
}

pub enum Response {
    PlainText(String),
    Dialog(slack::dialog::Dialog),
    AttachedMessage(slack::AttachedMessage),
    Json(serde_json::Value),
}

pub fn handle_command(
    command: &str,
    data: slack::slash_command::Request,
) -> Result<serde_json::Value, Box<std::error::Error>> {
    if SETTINGS.read()?.verification_token != data.token {
        return Err(From::from("Invalid token".to_owned()));
    }

    let result = match command {
        "add" => add_command(),
        "get" => get_command(&data.text),
        "edit" => edit_command(&data.text),
        "list" => list_command(&data.text),
        "issue" => issue_command(&data.text),
        _ => Err(From::from(format!("No such command: {}", command))),
    }?;

    match result {
        Response::PlainText(t) => Ok(json!({ "text": t })),
        Response::Dialog(d) => {
            show_dialog(&SETTINGS.read()?.api_token, d, &data.trigger_id)?;
            Ok(json!({ "text": "Dialog opened!" }))
        }
        Response::AttachedMessage(m) => Ok(serde_json::to_value(m)?),
        Response::Json(j) => Ok(j),
    }
}

pub fn handle_submission(
    submission: slack::dialog::Submission,
) -> Result<(), Box<std::error::Error>> {
    if SETTINGS.read()?.verification_token != submission.token {
        return Err(From::from("Invalid token".to_owned()));
    }
    if submission.submission_type != "dialog_submission" {
        return Err(From::from("Invalid submission".to_owned()));
    }

    match submission.callback_id.as_ref() {
        "add" => add_submission(submission),
        "edit" => edit_submission(submission),
        _ => Err(From::from(
            format!("No such submission: {}", submission.callback_id),
        )),
    }
}

fn add_command() -> Result<Response, Box<std::error::Error>> {
    Ok(Response::Dialog(generate_add_dialog()))
}

fn get_command(query: &str) -> Result<Response, Box<std::error::Error>> {
    use ip::get;
    if query.is_empty() {
        return Ok(Response::PlainText("Invalid argument".to_owned()));
    }
    let entry = get(query, &SETTINGS.read()?.data_path);
    match entry {
        Some(e) => Ok(Response::AttachedMessage(generate_get_message(e))),
        None => Ok(Response::PlainText("IP not found".to_owned())),
    }
}

fn edit_command(query: &str) -> Result<Response, Box<std::error::Error>> {
    use ip::get;
    if query.is_empty() {
        return Ok(Response::PlainText("Invalid argument".to_owned()));
    }
    let entry = match get(query, &SETTINGS.read()?.data_path) {
        None => return Ok(Response::PlainText("IP not found".to_owned())),
        Some(e) => e,
    };

    Ok(Response::Dialog(generate_edit_dialog(entry)))
}

fn list_command(query: &str) -> Result<Response, Box<std::error::Error>> {
    use ip::list;
    let entries = list(query, &SETTINGS.read()?.data_path);
    if entries.is_empty() {
        return Ok(Response::PlainText("IP not found".to_owned()));
    }
    Ok(Response::AttachedMessage(
        generate_list_message(query, entries),
    ))
}

fn issue_command(ports: &str) -> Result<Response, Box<std::error::Error>> {
    use ip::issue;
    match issue(
        &ports
            .split(' ')
            .filter_map(|p| p.parse::<u32>().ok())
            .collect::<Vec<u32>>(),
        &SETTINGS.read()?.data_path,
    ) {
        Some(e) => Ok(Response::Dialog(generate_edit_dialog(e))),
        None => Ok(Response::PlainText("No available IP".to_owned())),
    }
}

fn add_submission(submission: slack::dialog::Submission) -> Result<(), Box<std::error::Error>> {
    use ip::{add, Entry};
    let entry: Entry = submission.submission.into();
    add(&entry, &SETTINGS.read()?.data_path)?;
    Ok(())
}

fn edit_submission(submission: slack::dialog::Submission) -> Result<(), Box<std::error::Error>> {
    use ip::{add, Entry};
    let entry: Entry = submission.submission.into();
    add(&entry, &SETTINGS.read()?.data_path)?;
    Ok(())
}

fn generate_get_message(entry: ip::Entry) -> slack::AttachedMessage {
    use slack::*;
    let mut m = AttachedMessage {
        attachments: vec![],
    };
    let mut a = Attachment {
        title: format!("IP {}의 정보", entry.ip),
        fields: vec![],
    };
    let joined_ports = entry.ports_as_string();

    a.fields.push(AttachmentFields {
        title: "IP".to_owned(),
        value: entry.ip,
    });
    if let Some(domain) = entry.domain {
        a.fields.push(AttachmentFields {
            title: "도메인".to_owned(),
            value: domain,
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
        a.fields.push(AttachmentFields {
            title: "개방 포트".to_owned(),
            value: joined_ports,
        });
    }
    if let Some(description) = entry.description {
        a.fields.push(AttachmentFields {
            title: "설명".to_owned(),
            value: description,
        });
    }
    m.attachments.push(a);
    m
}

fn generate_list_message(query: &str, queries: Vec<ip::Query>) -> slack::AttachedMessage {
    use slack::*;
    let mut m = AttachedMessage {
        attachments: vec![],
    };
    let mut a = Attachment {
        title: if query.is_empty() {
            "IP 목록".to_owned()
        } else {
            format!("{}의 검색 결과", query)
        },
        fields: vec![],
    };

    for q in queries {
        a.fields.push(AttachmentFields {
            title: q.ip,
            value: q.element,
        });
    }
    m.attachments.push(a);
    m
}

fn show_dialog(
    token: &str,
    dialog: slack::dialog::Dialog,
    trigger_id: &str,
) -> Result<(), Box<std::error::Error>> {
    let request = slack::dialog::OpenRequest {
        token: token.to_owned(),
        dialog,
        trigger_id: trigger_id.to_owned(),
    };
    slack::dialog::open(request)?;
    Ok(())
}

fn generate_add_dialog() -> slack::dialog::Dialog {
    let mut dialog = slack::dialog::Dialog::new("add".to_owned(), "IP 추가".to_owned());

    dialog
        .elements
        .push(generate_ip_text(None).into_json().unwrap());
    dialog
        .elements
        .push(generate_domain_text(None).into_json().unwrap());
    dialog
        .elements
        .push(generate_using_select(None).into_json().unwrap());
    dialog
        .elements
        .push(generate_open_ports_text(None).into_json().unwrap());
    dialog
        .elements
        .push(generate_description_textarea(None).into_json().unwrap());

    dialog
}

fn generate_edit_dialog(entry: ip::Entry) -> slack::dialog::Dialog {
    let mut dialog = slack::dialog::Dialog::new("edit".to_owned(), "IP 수정".to_owned());
    let joined_ports = entry.ports_as_string();

    dialog
        .elements
        .push(generate_ip_text(Some(entry.ip)).into_json().unwrap());
    dialog
        .elements
        .push(generate_domain_text(entry.domain).into_json().unwrap());
    dialog.elements.push(
        generate_using_select(Some(if entry.using { "true" } else { "false" }.to_owned()))
            .into_json()
            .unwrap(),
    );
    dialog.elements.push(
        generate_open_ports_text({
            if !entry.open_ports.is_empty() {
                Some(joined_ports)
            } else {
                None
            }
        }).into_json()
            .unwrap(),
    );
    dialog.elements.push(
        generate_description_textarea(entry.description)
            .into_json()
            .unwrap(),
    );

    dialog
}

fn generate_ip_text(value: Option<String>) -> slack::dialog::element::Element {
    use slack::dialog::element::{Element, TextElement};
    Element::Text(TextElement {
        label: "IP".to_owned(),
        name: "ip".to_owned(),
        optional: None,
        hint: None,
        subtype: None,
        value,
        placeholder: None,
    })
}

fn generate_domain_text(value: Option<String>) -> slack::dialog::element::Element {
    use slack::dialog::element::{Element, TextElement};
    Element::Text(TextElement {
        label: "도메인".to_owned(),
        name: "domain".to_owned(),
        optional: Some(true),
        hint: None,
        subtype: None,
        value,
        placeholder: Some("Optional".to_owned()),
    })
}

fn generate_using_select(value: Option<String>) -> slack::dialog::element::Element {
    use slack::dialog::element::{Element, SelectElement, SelectOption};
    Element::Select(SelectElement {
        label: "사용 여부".to_owned(),
        name: "using".to_owned(),
        optional: None,
        options: vec![
            SelectOption {
                label: "사용중".to_owned(),
                value: "true".to_owned(),
            },
            SelectOption {
                label: "미사용".to_owned(),
                value: "false".to_owned(),
            },
        ],
        value,
        placeholder: None,
    })
}

fn generate_open_ports_text(value: Option<String>) -> slack::dialog::element::Element {
    use slack::dialog::element::{Element, TextElement};
    Element::Text(TextElement {
        label: "개방된 포트".to_owned(),
        name: "open_ports".to_owned(),
        optional: Some(true),
        hint: Some("쉼표로 구분".to_owned()),
        subtype: None,
        value,
        placeholder: Some("ex) 22, 80".to_owned()),
    })
}

fn generate_description_textarea(value: Option<String>) -> slack::dialog::element::Element {
    use slack::dialog::element::{Element, TextAreaElement};
    Element::TextArea(TextAreaElement {
        label: "설명".to_owned(),
        name: "description".to_owned(),
        optional: Some(true),
        hint: None,
        subtype: None,
        value,
        placeholder: Some("Optional".to_owned()),
    })
}
