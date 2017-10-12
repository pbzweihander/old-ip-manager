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

pub fn issue(required_ports: &[u32]) -> Result<Entry, Box<::std::error::Error>> {
    use std::fs::{read_dir, DirEntry, File, ReadDir};
    use std::io::Read;

    let dir_entries: ReadDir = read_dir("data")?;
    let files: Vec<DirEntry> = dir_entries.map(|e| e.unwrap()).collect();
    let entry = files
        .into_iter()
        .map(|f| {
            let mut file = File::open(f.path()).unwrap();
            let mut content: String = String::new();
            file.read_to_string(&mut content).unwrap();
            toml::from_str::<Entry>(&content).unwrap()
        })
        .find(|e| {
            !e.using && required_ports.into_iter().all(|p| e.open_ports.contains(p))
        });
    match entry {
        Some(e) => Ok(e),
        None => Err(Box::new(::std::io::Error::new(
            ::std::io::ErrorKind::Other,
            "No available IP",
        ))),
    }
}
