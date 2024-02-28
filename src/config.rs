mod sys;
use super::trie::Trie;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use xcfg_rs::File as ConfigFile;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Appattr {
    pub dir: String,
    pub exec: String,
    pub freq: u32,
    pub args: String,
    pub arg_hint: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    pub apps: HashMap<String, Appattr>,
}
impl Default for File {
    fn default() -> Self {
        Self {
            apps: HashMap::default(),
        }
    }
}
#[derive(Debug)]
pub struct App {
    pub name: String,
    pub id: u32,
    pub attr: Appattr,
}
#[derive(Debug)]
pub struct Config {
    pub trie: Trie<Arc<App>>,
    pub by_id: HashMap<u32, Arc<App>>,
    pub file: xcfg_rs::File<File>,
}

impl Config {
    pub fn new() -> Config {
        let path = dirs::home_dir()
            .unwrap()
            .join(".config/qst/appsearcher.toml");
        let mut file: ConfigFile<File> = ConfigFile::new().path(path.to_str().unwrap());
        let _ = file.load();
        sys::update_system(&mut file.inner.apps);
        let mut id: u32 = 0;
        let mut trie = Trie::new();
        let mut by_id = HashMap::new();
        file.inner
            .apps
            .clone()
            .into_iter()
            .for_each(|(name, attr)| {
                let app = Arc::new(App { name, id, attr });
                trie.insert(app.name.clone(), app.clone());
                by_id.insert(id, app);
                id += 1;
            });
        Self { by_id, trie, file }
    }

    pub fn save(&self) {
        self.file.save();
    }
}
