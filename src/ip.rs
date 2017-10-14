extern crate serde_derive;
extern crate toml;

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub ip: String,
    pub domain: Option<String>,
    pub using: bool,
    pub open_ports: Vec<u32>,
    pub description: Option<String>,
}

impl Entry {
    pub fn ports_as_string(&self) -> String {
        let mut s = String::new();
        for p in &self.open_ports {
            s.push_str(&format!("{}, ", p));
        }
        s.pop();
        s.pop();
        s
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawEntry {
    pub ip: String,
    pub domain: Option<String>,
    pub using: String,
    pub open_ports: Option<String>,
    pub description: Option<String>,
}

pub struct Query {
    pub ip: String,
    pub element: String,
}

impl ::std::convert::Into<Entry> for RawEntry {
    fn into(self) -> Entry {
        let using = self.using == "true";
        let open_ports = match self.open_ports {
            None => vec![],
            Some(s) => if !s.is_empty() {
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

pub fn add(entry: &Entry) -> Result<(), Box<::std::error::Error>> {
    use std::fs::File;
    use std::path::Path;
    use std::io::Write;

    let s = toml::to_string_pretty(entry)?;
    let spath = format!("data/{}.toml", entry.ip);
    let p = Path::new(&spath);
    let mut file: File = File::create(&p)?;
    file.write_all(s.as_bytes())?;

    Ok(())
}

pub fn get(ip: &str) -> Option<Entry> {
    use std::fs::File;
    use std::path::Path;
    use std::io::Read;

    let spath = format!("data/{}.toml", ip);
    let p = Path::new(&spath);
    let mut file: File = match File::open(&p) {
        Ok(f) => f,
        Err(_) => return None,
    };
    let mut content = String::new();
    if file.read_to_string(&mut content).is_err() {
        return None;
    }
    toml::from_str(&content).ok()
}

pub fn list(query: &str) -> Vec<Query> {
    use std::fs::{read_dir, DirEntry, File, ReadDir};
    use std::io::Read;

    let dir_entries: ReadDir = match read_dir("data") {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    let files: Vec<DirEntry> = dir_entries
        .filter(|e| e.is_ok())
        .map(|e| e.unwrap())
        .collect();
    let entries = files.into_iter().map(|f| {
        let mut file = File::open(f.path()).unwrap();
        let mut content: String = String::new();
        file.read_to_string(&mut content).unwrap();
        toml::from_str::<Entry>(&content).unwrap()
    });
    let entries: Vec<Query> = if !query.is_empty() {
        entries
            .filter_map(|e| {
                query
                    .split(' ')
                    .filter(|q| !q.is_empty())
                    .filter_map(|q| generate_query(&e, q))
                    .next()
            })
            .take(8)
            .collect()
    } else {
        entries
            .map(|e| {
                Query {
                    ip: e.ip.clone(),
                    element: {
                        let mut s = String::new();
                        if let Some(ref domain) = e.domain {
                            s.push_str(domain);
                            s.push_str("\n");
                        }
                        s.push_str(if e.using { "사용중" } else { "미사용" });
                        s
                    },
                }
            })
            .take(8)
            .collect()
    };
    entries
}

fn generate_query(entry: &Entry, q: &str) -> Option<Query> {
    if entry.ip.contains(q) {
        return Some(Query {
            ip: entry.ip.clone(),
            element: {
                let mut s = String::new();
                if let Some(ref domain) = entry.domain {
                    s.push_str(domain);
                    s.push_str("\n");
                }
                s.push_str(if entry.using {
                    "사용중"
                } else {
                    "미사용"
                });
                s
            },
        });
    }
    if let Some(ref domain) = entry.domain {
        if domain.contains(q) {
            return Some(Query {
                ip: entry.ip.clone(),
                element: entry.domain.as_ref().unwrap().clone(),
            });
        }
    }
    if entry.using && q == "사용중" {
        return Some(Query {
            ip: entry.ip.clone(),
            element: "사용중".to_owned(),
        });
    }
    if !entry.using && q == "미사용" {
        return Some(Query {
            ip: entry.ip.clone(),
            element: "미사용".to_owned(),
        });
    }
    if let Ok(i) = q.parse::<u32>() {
        if entry.open_ports.contains(&i) {
            return Some(Query {
                ip: entry.ip.clone(),
                element: entry.ports_as_string(),
            });
        }
    }
    if entry.description.is_some() && entry.description.as_ref().unwrap().contains(q) {
        return Some(Query {
            ip: entry.ip.clone(),
            element: entry.description.as_ref().unwrap().clone(),
        });
    }
    None
}

pub fn issue(required_ports: &[u32]) -> Option<Entry> {
    use std::fs::{read_dir, DirEntry, File, ReadDir};
    use std::io::Read;

    let dir_entries: ReadDir = match read_dir("data") {
        Ok(r) => r,
        Err(_) => return None,
    };
    let files: Vec<DirEntry> = dir_entries
        .filter(|e| e.is_ok())
        .map(|e| e.unwrap())
        .collect();
    files
        .into_iter()
        .map(|f| {
            let mut file = File::open(f.path()).unwrap();
            let mut content: String = String::new();
            file.read_to_string(&mut content).unwrap();
            toml::from_str::<Entry>(&content).unwrap()
        })
        .find(|e| {
            !e.using
                && (required_ports.is_empty()
                    || required_ports.into_iter().all(|p| e.open_ports.contains(p)))
        })
}
