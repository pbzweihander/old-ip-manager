extern crate serde_derive;

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub ip: String,
    pub domain: Option<String>,
    pub using: bool,
    pub open_ports: Vec<u32>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawEntry {
    pub ip: String,
    pub domain: Option<String>,
    pub using: String,
    pub open_ports: Option<String>,
    pub description: Option<String>,
}

impl ::std::convert::Into<Entry> for RawEntry {
    fn into(self) -> Entry {
        let using = match self.using.as_ref() {
            "true" => true,
            "false" => false,
            _ => false,
        };
        let open_ports = match self.open_ports {
            None => vec![],
            Some(s) => if s.len() > 0 {
                s.split(',')
                    .map(|s| s.trim())
                    .map(|s| s.parse::<u32>())
                    .filter(|s| s.is_ok())
                    .map(|s| s.unwrap())
                    .collect()
            } else {
                vec![]
            },
        };
        Entry {
            ip: self.ip,
            domain: self.domain,
            using,
            open_ports,
            description: self.description,
        }
    }
}

#[derive(Default)]
pub struct List {
    hashmap: ::std::collections::HashMap<String, Entry>,
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
            !e.using
                && (&required_ports)
                    .into_iter()
                    .all(|p| e.open_ports.contains(p))
        })
    }
}
