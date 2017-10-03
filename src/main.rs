#![feature(plugin, custom_derive, decl_macro)]
#![plugin(rocket_codegen)]
extern crate ip_manager;
extern crate rocket;
extern crate rocket_contrib;
extern crate toml;

use std::fs;
use std::io::Read;
use std::sync::{Arc, Mutex};
use rocket::request::LenientForm;
use rocket::State;
use ip_manager::{ip, settings, slash_command};

enum Command {
    Add,
    Get,
    Edit,
    Issue,
    Help,
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Command::*;
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

fn main() {
    try_main().unwrap();
}

fn try_main() -> Result<(), Box<std::error::Error>> {
    let ip_list = Arc::new(Mutex::new(ip::List::default()));
    let settings = Arc::new(Mutex::new(settings::Settings::new()?));

    let dir_entries: fs::ReadDir = fs::read_dir("data")?;
    let files: Vec<fs::DirEntry> = dir_entries.map(|e| e.unwrap()).collect();
    {
        let mut ips = ip_list.lock().unwrap();
        files
            .into_iter()
            .map(|f| {
                let mut file = fs::File::open(f.path()).unwrap();
                let mut content: String = String::new();
                file.read_to_string(&mut content).unwrap();
                toml::from_str(&content).unwrap()
            })
            .for_each(|e| ips.add(e));
    }

    rocket::ignite()
        .manage(ip_list)
        .manage(settings)
        .mount("/ip-manager", routes![command_request])
        .launch();
    Ok(())
}

#[post("/command", data = "<form>")]
fn command_request(
    form: LenientForm<slash_command::Request>,
    ips: State<Arc<Mutex<ip::List>>>,
    settings: State<Arc<Mutex<settings::Settings>>>,
) -> String {
    let data = form.into_inner();
    let s: &settings::Settings = &*settings.lock().unwrap();
    if s.verification_token != data.token {
        return "".to_owned();
    }

    let command = parse_command(&data.text);
    let i = &*ips.lock().unwrap();
    do_command(&command, i, &data.text)
}

fn parse_command(content: &str) -> Command {
    use Command::*;
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
    use Command::*;
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
