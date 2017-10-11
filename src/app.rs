extern crate serde_json;

use super::ip;
use super::slack::slash_command;
use super::slack::dialog::{open, Dialog, OpenRequest, Submission};
use super::slack::dialog::element::*;
use super::settings;
use std::error::Error;

enum Command {
    Add,
    Get,
    Edit,
    Issue,
    Help,
}

impl ::std::fmt::Display for Command {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use self::Command::*;
        write!(
            f,
            "{}",
            match *self {
                Add => "Add",
                Get => "Get",
                Edit => "Edit",
                Issue => "Issue",
                Help => "Help",
            }
        )
    }
}

pub struct App {
    ip_list: ip::List,
    settings: settings::Settings,
}

impl App {
    pub fn new(ip_list: ip::List, settings: settings::Settings) -> App {
        App { ip_list, settings }
    }

    pub fn handle_command(&self, data: slash_command::Request) -> String {
        if !self.validate(&data.token) {
            return "".to_owned();
        }

        let command = parse_command(&data.text);
        do_command(&command, &self.settings, &self.ip_list, data)
    }

    pub fn handle_submission(&self, submission: Submission) -> String {
        "".to_owned()
    }

    fn validate(&self, token: &str) -> bool {
        token == self.settings.verification_token
    }
}

fn parse_command(content: &str) -> Command {
    use self::Command::*;
    match content.split(' ').nth(0) {
        Some(s) => match &s.to_lowercase()[..] {
            "add" => Add,
            "get" => Get,
            "edit" => Edit,
            "issue" => Issue,
            _ => Help,
        },
        None => Help,
    }
}

fn do_command(
    command: &Command,
    settings: &settings::Settings,
    ips: &ip::List,
    data: slash_command::Request,
) -> String {
    use self::Command::*;
    match match *command {
        Add => add_command(settings, ips, data),
        Get => get_command(settings, ips, data),
        Edit => edit_command(settings, ips, data),
        Issue => issue_command(settings, ips, data),
        Help => help_command(settings, ips, data),
    } {
        Ok(s) => s,
        Err(e) => format!("Error: {}", e),
    }
}

fn add_command(
    settings: &settings::Settings,
    _ips: &ip::List,
    data: slash_command::Request,
) -> Result<String, Box<Error>> {
    let mut dialog = Dialog::new("add".to_owned(), "IP 추가".to_owned());

    let ip = Text {
        label: "IP".to_owned(),
        name: "ip".to_owned(),
        optional: None,
        hint: None,
        subtype: None,
        value: None,
        placeholder: None,
    };
    dialog.elements.push(ip.into_json()?);

    let domain = Text {
        label: "도메인".to_owned(),
        name: "domain".to_owned(),
        optional: Some(true),
        hint: None,
        subtype: None,
        value: None,
        placeholder: Some("Optional".to_owned()),
    };
    dialog.elements.push(domain.into_json()?);

    let using = Select {
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
        value: "false".to_owned(),
        placeholder: None,
    };
    dialog.elements.push(using.into_json()?);

    let open_ports = Text {
        label: "개방된 포트".to_owned(),
        name: "open_ports".to_owned(),
        optional: Some(true),
        hint: Some("쉼표로 구분".to_owned()),
        subtype: None,
        value: None,
        placeholder: Some("ex) 22, 80".to_owned()),
    };
    dialog.elements.push(open_ports.into_json()?);

    let description = TextArea {
        label: "설명".to_owned(),
        name: "description".to_owned(),
        optional: Some(true),
        hint: None,
        subtype: None,
        value: None,
        placeholder: Some("Optional".to_owned()),
    };
    dialog.elements.push(description.into_json()?);

    let request = OpenRequest {
        token: settings.api_token.clone(),
        dialog,
        trigger_id: data.trigger_id,
    };
    open(request)?;

    Ok("".to_owned())
}

fn get_command(
    settings: &settings::Settings,
    ips: &ip::List,
    data: slash_command::Request,
) -> Result<String, Box<Error>> {
    Ok("get".to_owned())
}

fn edit_command(
    settings: &settings::Settings,
    ips: &ip::List,
    data: slash_command::Request,
) -> Result<String, Box<Error>> {
    Ok("edit".to_owned())
}

fn issue_command(
    settings: &settings::Settings,
    ips: &ip::List,
    data: slash_command::Request,
) -> Result<String, Box<Error>> {
    Ok("issue".to_owned())
}

fn help_command(
    settings: &settings::Settings,
    ips: &ip::List,
    data: slash_command::Request,
) -> Result<String, Box<Error>> {
    Ok("help".to_owned())
}
