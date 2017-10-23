extern crate config;
extern crate reqwest;
extern crate serde_json;
extern crate toml;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Config(config::ConfigError);
        Json(serde_json::error::Error);
        Request(reqwest::Error);
        TomlSerialize(toml::ser::Error);
    }
    errors {
        Poisoned(a: &'static str) {
            description("lock poisoned")
            display("Lock poisoned at {}", a)
        }
        InvalidToken {
            description("invalid token")
            display("Invalid token")
        }
        InvalidSubmission {
            description("invalid submission")
            display("Invalid submission")
        }
        CommandNotFound(c: String) {
            description("command not found")
            display("No such command: {}", c)
        }
        SubmissionNotFound(c: String) {
            description("submission not found")
            display("No such submission: {}", c)
        }
    }
}
