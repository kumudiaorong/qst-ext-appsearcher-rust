mod sys;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use xcfg::File as XFile;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Appattr {
    pub dir: String,
    pub exec: String,
    pub freq: u32,
    pub args: String,
    pub arg_hint: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    pub apps: HashMap<String, Appattr>,
}
impl Default for Info {
    fn default() -> Self {
        Self {
            apps: HashMap::default(),
        }
    }
}
pub type Config = XFile<Info>;

pub fn init() -> Config {
    let path = dirs::home_dir()
        .unwrap()
        .join(".config/qst/appsearcher.toml");
    let mut file: XFile<Info> = XFile::new().path(path.to_str().unwrap());
    let _ = file.load();
    let _ = file.save();
    sys::update_system(&mut file.inner.apps);
    file
}
