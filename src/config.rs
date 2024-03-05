mod sys;
use crate::trie::Trie;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
};
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
    pub file: XFile<Info>,
}

impl Config {
    pub fn new() -> Config {
        let path = dirs::home_dir()
            .unwrap()
            .join(".config/qst/appsearcher.toml");
        let mut file: XFile<Info> = XFile::new().path(path.to_str().unwrap());
        let _ = file.load();
        let _ = file.save();
        sys::update_system(&mut file.inner.apps);
        let mut id: u32 = 0;
        let mut trie = Trie::new();
        let mut by_id = HashMap::new();
        file.inner.apps.iter().for_each(|(name, attr)| {
            let app = Arc::new(App {
                name: name.clone(),
                id,
                attr: attr.clone(),
            });
            trie.insert(app.name.clone(), app.clone());
            by_id.insert(id, app);
            id += 1;
        });
        Self { by_id, trie, file }
    }
    pub fn save(&self) {
        self.file.save().unwrap();
    }
}
