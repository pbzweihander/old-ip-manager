#![feature(plugin, custom_derive, decl_macro)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

pub mod app;

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
