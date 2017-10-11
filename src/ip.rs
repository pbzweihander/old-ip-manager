extern crate serde_derive;

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
