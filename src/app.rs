use super::ip;
use super::slash_command;
use super::settings;

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
        if !self.validate(data.token) {
            return "".to_owned();
        }

        let command = parse_command(&data.text);
        do_command(&command, &self.ip_list, &data.text)
    }

    fn validate(&self, token: String) -> bool {
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

fn do_command(command: &Command, ips: &ip::List, content: &str) -> String {
    use self::Command::*;
    match *command {
        Add => add(ips, content),
        Get => get(ips, content),
        Edit => edit(ips, content),
        Issue => issue(ips, content),
        Help => help(ips, content),
    }
}

fn add(ips: &ip::List, content: &str) -> String {
    "add".to_owned()
}

fn get(ips: &ip::List, content: &str) -> String {
    "get".to_owned()
}

fn edit(ips: &ip::List, content: &str) -> String {
    "edit".to_owned()
}

fn issue(ips: &ip::List, content: &str) -> String {
    "issue".to_owned()
}

fn help(ips: &ip::List, content: &str) -> String {
    "help".to_owned()
}
