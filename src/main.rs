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
        write!(
            f,
            "{}",
            match self {
                &Command::Add => "Add",
                &Command::Get => "Get",
                &Command::Edit => "Edit",
                &Command::Issue => "Issue",
                &Command::Help => "Help",
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
fn command_request(form: LenientForm<slash_command::Request>) -> String {
    let data = form.into_inner();
    let command = parse_command(&data.text);
    println!("{}", command);
    "foo".to_owned()
}

fn parse_command(content: &str) -> Command {
    match content.split(' ').nth(0) {
        Some(s) => match &s.to_lowercase()[..] {
            "add" => Command::Add,
            "get" => Command::Get,
            "edit" => Command::Edit,
            "issue" => Command::Issue,
            _ => Command::Help,
        },
        None => Command::Help,
    }
}
