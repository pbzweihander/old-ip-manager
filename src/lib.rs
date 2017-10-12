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
    Text(String),
    Json(serde_json::Value),
}

pub fn handle_command(
    settings: &settings::Settings,
    command: &str,
    data: slack::slash_command::Request,
) -> Result<serde_json::Value, Box<std::error::Error>> {
    if settings.verification_token != data.token {
        return Ok(json!({"text": "Validation error".to_owned()}));
    }

    let result = match command {
        "add" => add_command(settings, data),
        "get" => get_command(&data),
        _ => Ok(Response::Text("Nope".to_owned())),
    }?;

    match result {
        Response::Text(t) => Ok(json!({ "text": t })),
        Response::Json(j) => Ok(j),
    }
}

pub fn handle_submission(
    settings: &settings::Settings,
    submission: slack::dialog::Submission,
) -> Result<String, Box<std::error::Error>> {
    if settings.verification_token != submission.token {
        return Ok("Validation error".to_owned());
    }

    match submission.callback_id.as_ref() {
        "add" => add_submission(submission),
        _ => Ok("".to_owned()),
    }
}

fn add_command(
    settings: &settings::Settings,
    data: slack::slash_command::Request,
) -> Result<Response, Box<std::error::Error>> {
    show_add_dialog(settings, data)?;
    Ok(Response::Text("Dialog opend!".to_owned()))
}

fn show_add_dialog(
    settings: &settings::Settings,
    data: slack::slash_command::Request,
) -> Result<(), Box<std::error::Error>> {
    use slack::dialog::*;
    let mut dialog = Dialog::new("add".to_owned(), "IP 추가".to_owned());

    dialog.elements.push(generate_ip_text(None).into_json()?);
    dialog
        .elements
        .push(generate_domain_text(None).into_json()?);
    dialog
        .elements
        .push(generate_using_select(None).into_json()?);
    dialog
        .elements
        .push(generate_open_ports_text(None).into_json()?);
    dialog
        .elements
        .push(generate_description_textarea(None).into_json()?);

    let request = OpenRequest {
        token: settings.api_token.clone(),
        dialog,
        trigger_id: data.trigger_id,
    };
    open(request)?;

    Ok(())
}

fn get_command(data: &slack::slash_command::Request) -> Result<Response, Box<std::error::Error>> {
    use ip::get;
    if data.text.is_empty() {
        return Ok(Response::Text("Validation error".to_owned()));
    }
    let entry = get(&data.text);
    match entry {
        Some(e) => Ok(Response::Json(serde_json::to_value(show_get_info(&e))?)),
        None => Ok(Response::Text("IP not found".to_owned())),
    }
}

fn add_submission(submission: slack::dialog::Submission) -> Result<String, Box<std::error::Error>> {
    use ip::{add, Entry};
    let entry: Entry = submission.submission.into();
    add(&entry)?;
    Ok("".to_owned())
}

fn show_get_info(entry: &ip::Entry) -> slack::AttachedMessage {
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
