#![feature(plugin, custom_derive, decl_macro)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

pub mod app;

pub mod ip;

pub mod slash_command;

pub mod settings {
    extern crate config;

    #[derive(Deserialize)]
    pub struct Settings {
        pub verification_token: String,
        pub bot_token: String,
    }

    impl Settings {
        pub fn new() -> Result<Settings, config::ConfigError> {
            let mut settings = config::Config::new();
            settings.merge(config::File::with_name("settings"))?;
            settings.try_into::<Settings>()
        }
    }
}
