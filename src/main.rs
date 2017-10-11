#![feature(plugin, custom_derive, decl_macro)]
#![plugin(rocket_codegen)]
extern crate ip_manager;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde_json;
extern crate toml;

use std::sync::{Arc, Mutex};
use rocket::request::LenientForm;
use rocket::State;
use ip_manager::{app, settings};
use ip_manager::slack::*;
use rocket_contrib::Json;

fn main() {
    try_main().unwrap();
}

fn try_main() -> Result<(), Box<std::error::Error>> {
    let settings = settings::Settings::new()?;
    let app = app::App::new(settings);

    rocket::ignite()
        .manage(Arc::new(Mutex::new(app)))
        .mount("/ip-manager/command", routes![add_command, get_command])
        .mount("/ip-manager", routes![dialog_response])
        .launch();
    Ok(())
}

#[post("/add", data = "<form>")]
fn add_command(
    form: LenientForm<slash_command::Request>,
    app: State<Arc<Mutex<app::App>>>,
) -> Json {
    let data = form.into_inner();
    Json(app.lock().unwrap().handle_command(app::CommandType::Add, data))
}

#[post("/get", data = "<form>")]
fn get_command(
    form: LenientForm<slash_command::Request>,
    app: State<Arc<Mutex<app::App>>>,
) -> Json {
    let data = form.into_inner();
    Json(app.lock().unwrap().handle_command(app::CommandType::Get, data))
}

#[post("/submission", data = "<form>")]
fn dialog_response(
    form: LenientForm<dialog::SubmissionResponse>,
    app: State<Arc<Mutex<app::App>>>,
) -> String {
    let data: dialog::Submission = serde_json::from_str(&form.into_inner().payload).unwrap();
    app.lock().unwrap().handle_submission(data)
}
