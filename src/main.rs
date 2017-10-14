#![feature(plugin, custom_derive, decl_macro)]
#![plugin(rocket_codegen)]
extern crate ip_manager;
#[macro_use]
extern crate lazy_static;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde_json;

use std::sync::RwLock;
use rocket::request::LenientForm;
use ip_manager::settings::Settings;
use ip_manager::{handle_command, handle_submission};
use ip_manager::slack::slash_command::Request;
use ip_manager::slack::dialog::{Submission, SubmissionResponse};

lazy_static! {
    static ref SETTINGS: RwLock<Settings> = RwLock::new(Settings::new().unwrap());
}

fn main() {
    try_main().unwrap();
}

fn try_main() -> Result<(), Box<std::error::Error>> {
    validate_data_path();

    rocket::ignite()
        .mount("/ip-manager/command", routes![command_request])
        .mount("/ip-manager/submission", routes![dialog_response])
        .launch();
    Ok(())
}

#[post("/<command>", data = "<form>")]
fn command_request(
    command: String,
    form: LenientForm<Request>,
) -> Result<rocket_contrib::Json, Box<std::error::Error>> {
    let data = form.into_inner();
    let json = handle_command(&SETTINGS.read().unwrap(), &command, data)?;
    Ok(rocket_contrib::Json(json))
}

#[post("/", data = "<form>")]
fn dialog_response(
    form: LenientForm<SubmissionResponse>,
) -> Result<String, Box<std::error::Error>> {
    let data: Submission = serde_json::from_str(&form.into_inner().payload).unwrap();
    handle_submission(&SETTINGS.read().unwrap(), data)?;
    Ok("".to_owned())
}

fn validate_data_path() {
    use std::fs::read_dir;
    if read_dir(&SETTINGS.read().unwrap().data_path).is_err() {
        panic!("Invalid data folder. Check settings file!");
    }
}
