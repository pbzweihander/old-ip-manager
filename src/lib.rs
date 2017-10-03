#![feature(plugin, custom_derive, decl_macro)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

pub mod ip {
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize)]
    pub struct Entry {
        pub ip: String,
        pub mac: Option<String>,
        pub request_date: String,
        pub domain: Option<String>,
        pub description: Option<String>,
        pub using: bool,
        pub note: Option<String>,
        pub open_ports: Vec<u32>,
    }

    #[derive(Default)]
    pub struct List {
        hashmap: HashMap<String, Entry>,
    }

    impl List {
        pub fn add(&mut self, ip: Entry) {
            self.hashmap.insert(ip.ip.clone(), ip);
        }

        pub fn get(&self, ip: String) -> Option<&Entry> {
            self.hashmap.get(&ip)
        }

        pub fn issue(&self, required_ports: Vec<u32>) -> Option<&Entry> {
            self.hashmap.values().find(|e| {
                !e.using && e.mac.is_none()
                    && (&required_ports)
                        .into_iter()
                        .all(|p| e.open_ports.contains(p))
            })
        }
    }
}

pub mod slash_command {
    #[derive(FromForm)]
    pub struct Request {
        pub token: String,
        pub team_id: String,
        pub team_domain: String,
        pub channel_id: String,
        pub channel_name: String,
        pub user_id: String,
        pub user_name: String,
        pub text: String,
        pub response_url: String,
    }

    #[derive(Serialize)]
    pub struct Response {
        pub response_type: String,
        pub text: String,
        pub attachments: ResponseAttachments,
    }

    #[derive(Serialize)]
    pub struct ResponseAttachments {
        pub fallback: String,
        pub color: String,
        pub pretext: String,
        pub title: String,
        pub text: String,
        pub fields: Vec<AttachmentField>,
    }

    #[derive(Serialize)]
    pub struct AttachmentField {
        pub title: String,
        pub value: String,
        pub short: bool,
    }
}

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
