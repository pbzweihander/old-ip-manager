#![feature(plugin, custom_derive, decl_macro)]
#![plugin(rocket_codegen)]
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
    }

    impl Settings {
        pub fn new() -> Result<Settings, config::ConfigError> {
            let mut settings = config::Config::new();
            settings.merge(config::File::with_name("settings"))?;
            settings.try_into::<Settings>()
        }
    }
}

pub enum Response {
    PlainText(String),
    Dialog(slack::dialog::Dialog),
    AttachedMessage(slack::AttachedMessage),
    Json(serde_json::Value),
}

pub fn handle_command(
    settings: &settings::Settings,
    command: &str,
    data: slack::slash_command::Request,
) -> Result<serde_json::Value, Box<std::error::Error>> {
    if settings.verification_token != data.token {
        return Err(From::from("Validation error".to_owned()))
    }

    let result = match command {
        "add" => add_command(),
        "get" => get_command(&data.text),
        "edit" => edit_command(&data.text),
        _ => Err(From::from(format!("No such command: {}", command))),
    }?;

    match result {
        Response::PlainText(t) => Ok(json!({ "text": t })),
        Response::Dialog(d) => {
            show_dialog(&settings.api_token, d, &data.trigger_id)?;
            Ok(json!({ "text": "Dialog opened!" }))
        }
        Response::AttachedMessage(m) => Ok(serde_json::to_value(m)?),
        Response::Json(j) => Ok(j),
    }
}

pub fn handle_submission(
    settings: &settings::Settings,
    submission: slack::dialog::Submission,
) -> Result<(), Box<std::error::Error>> {
    if settings.verification_token != submission.token {
        return Err(From::from("Validation error".to_owned()))
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
        return Ok(Response::PlainText("Validation error".to_owned()));
    }
    let entry = get(query);
    match entry {
        Some(e) => Ok(Response::AttachedMessage(generate_get_message(e))),
        None => Ok(Response::PlainText("IP not found".to_owned())),
    }
}

fn edit_command(query: &str) -> Result<Response, Box<std::error::Error>> {
    use ip::get;
    if query.is_empty() {
        return Ok(Response::PlainText("Validation error".to_owned()));
    }
    let entry = match get(query) {
        None => return Ok(Response::PlainText("IP not found".to_owned())),
        Some(e) => e,
    };

    Ok(Response::Dialog(generate_edit_dialog(entry)))
}

fn add_submission(submission: slack::dialog::Submission) -> Result<(), Box<std::error::Error>> {
    use ip::{add, Entry};
    let entry: Entry = submission.submission.into();
    add(&entry)?;
    Ok(())
}

fn edit_submission(submission: slack::dialog::Submission) -> Result<(), Box<std::error::Error>> {
    use ip::{add, Entry};
    let entry: Entry = submission.submission.into();
    add(&entry)?;
    Ok(())
}

fn generate_get_message(entry: ip::Entry) -> slack::AttachedMessage {
    use slack::*;
    let mut m = AttachedMessage {
        attachments: vec![],
    };
    let mut a = Attachment { fields: vec![] };

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
        let mut s = String::new();
        for p in entry.open_ports {
            s.push_str(&format!("{}, ", p));
        }
        s.pop();
        s.pop();
        a.fields.push(AttachmentFields {
            title: "개방 포트".to_owned(),
            value: s,
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
                let mut s = String::new();
                for p in entry.open_ports {
                    s.push_str(&format!("{}, ", p));
                }
                s.pop();
                s.pop();
                Some(s)
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
