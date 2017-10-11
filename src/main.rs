#![feature(plugin, custom_derive, decl_macro)]
#![plugin(rocket_codegen)]
extern crate ip_manager;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde_json;
extern crate toml;

use std::fs;
use std::io::Read;
use std::sync::{Arc, Mutex};
use rocket::request::LenientForm;
use rocket::State;
use ip_manager::{app, ip, settings};
use ip_manager::slack::*;

fn main() {
    try_main().unwrap();
}

fn try_main() -> Result<(), Box<std::error::Error>> {
    let mut ip_list = ip::List::default();
    let settings = settings::Settings::new()?;

    let dir_entries: fs::ReadDir = fs::read_dir("data")?;
    let files: Vec<fs::DirEntry> = dir_entries.map(|e| e.unwrap()).collect();
    files
        .into_iter()
        .map(|f| {
            let mut file = fs::File::open(f.path()).unwrap();
            let mut content: String = String::new();
            file.read_to_string(&mut content).unwrap();
            toml::from_str(&content).unwrap()
        })
        .for_each(|e| ip_list.add(e));

    let app = app::App::new(ip_list, settings);

    rocket::ignite()
        .manage(Arc::new(Mutex::new(app)))
        .mount("/ip-manager", routes![command_request])
        .mount("/ip-manager", routes![dialog_response])
        .launch();
    Ok(())
}

#[post("/command", data = "<form>")]
fn command_request(
    form: LenientForm<slash_command::Request>,
    app: State<Arc<Mutex<app::App>>>,
) -> String {
    let data = form.into_inner();
    app.lock().unwrap().handle_command(data)
}

#[post("/submission", data = "<form>")]
fn dialog_response(
    form: LenientForm<dialog::SubmissionResponse>,
    app: State<Arc<Mutex<app::App>>>,
) -> String {
    let data: dialog::Submission = serde_json::from_str(&form.into_inner().payload).unwrap();
    app.lock().unwrap().handle_submission(data)
}
