use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use xlog_rs::log;

fn cfg_dir() -> Result<String, std::env::VarError> {
    std::option_env!("HOME")
        .ok_or(std::env::VarError::NotPresent)
        .map(|s| s.to_string() + "/.config/qst")
}

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
    pub system_last_update: std::time::SystemTime,
    pub apps: HashMap<String, Appattr>,
}
#[derive(Debug)]
pub struct App {
    pub name: String,
    pub id: u32,
    pub attr: Appattr,
}
#[derive(Debug)]
pub struct Config {
    pub system_last_update: std::time::SystemTime,
    pub file_last_update: Option<std::time::SystemTime>,
    pub trie: super::trie::Trie<Arc<App>>,
    pub by_id: HashMap<u32, Arc<App>>,
}

// fn extract_app(path: String) -> Option<(String, Appattr)> {
//     std::fs::read_to_string(path.clone()).map_or_else(
//         |e| {
//             log::warn(format!("can not read {}: {}", path, e).as_str());
//             None
//         },
//         |content: String| {
//             RE.captures(&content).map(|captures| {
//                 let exec = captures["exec"].to_string();
//                 (
//                     captures["name"].to_string(),
//                     Appattr {
//                         dir: exec
//                             .rsplit_once('/')
//                             .map_or("/usr/local/bin", |(dir, _)| dir)
//                             .to_string(),
//                         exec,
//                         freq: 0,
//                     },
//                 )
//             })
//         },
//     )
// }
fn extract_app(content: &str) -> Option<(String, Appattr)> {
    static FILE: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
        regex::Regex::new(
        r"\[Desktop Entry\][\S\s]*?(?:(?:\r|\n|\r\n)(?:(?:Exec\s?=\s?(?P<exec>.*[\S]))|(?:Name\s?=\s?(?P<name>.*[\S])))[\S\s]*?){2}"
    ).unwrap()
    });
    static CODE: once_cell::sync::Lazy<regex::Regex> =
        once_cell::sync::Lazy::new(|| regex::Regex::new(r"%(?<code>[fFuUick])").unwrap());
    FILE.captures(&content).map(|captures| {
        // log::info(format!("extracting app: {}", &captures["name"]).as_str());
        let cmd: &str = &captures["exec"];
        let argv = cmd.split(' ').collect::<Vec<_>>();
        let exec = argv[0].to_string();
        (
            captures["name"].to_string(),
            Appattr {
                dir: exec
                    .rsplit_once('/')
                    .map_or("/usr/local/bin", |(dir, _)| dir)
                    .to_string(),
                exec,
                freq: 0,
                args: argv[1..].join(" "),
                arg_hint: CODE.captures(cmd).map(|captures| {
                    match &captures["code"] {
                        "f" => "file",
                        "F" => "files",
                        "u" => "url",
                        "U" => "urls",
                        _ => "todo",
                    }
                    .to_string()
                }),
            },
        )
    })
}
#[test]
pub fn reg_test() {
    assert_eq!(
        extract_app(
            "[Desktop Entry]
Name=Visual Studio Code
Comment=Code Editing. Redefined.
GenericName=Text Editor
Exec=/usr/share/code/code --unity-launch %F
Icon=vscode
Type=Application
StartupNotify=false
StartupWMClass=Code
Categories=TextEditor;Development;IDE;
MimeType=text/plain;inode/directory;application/x-code-workspace;
Actions=new-empty-window;
Keywords=vscode;

[Desktop Action new-empty-window]
Name=New Empty Window
Exec=/usr/share/code/code --new-window %F
Icon=vscode
"
        )
        .unwrap(),
        (
            "Visual Studio Code".to_string(),
            Appattr {
                dir: "/usr/share/code".to_string(),
                exec: "/usr/share/code/code".to_string(),
                freq: 0,
                args: "--unity-launch %F".to_string(),
                arg_hint: Some("files".to_string()),
            }
        )
    );
}
fn update_system(src: &mut HashMap<String, Appattr>) {
    match std::fs::read_dir("/usr/share/applications") {
        Ok(rd) => {
            for entry in rd {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }
                let a = std::fs::read_to_string(path.clone()).map_or_else(
                    |e| {
                        log::warn(
                            format!("can not read {}: {}", path.to_str().unwrap(), e).as_str(),
                        );
                        None
                    },
                    |content: String| extract_app(&content),
                );
                if let Some((name, app)) = a {
                    src.entry(name)
                        .and_modify(|e| {
                            *e = app.clone();
                        })
                        .or_insert(app.clone());
                } else {
                    log::warn(format!("can not extract {}", path.to_str().unwrap()).as_str());
                }
            }
        }
        Err(_) => {
            log::warn("can not read /usr/share/applications");
        }
    };
}

impl Config {
    pub fn new() -> Config {
        let dir = cfg_dir().expect("Failed to get config directory");
        let (mut file, sf) = match std::fs::File::open(dir.clone() + "/appsearcher.yaml") {
            Ok(sf) => (serde_yaml::from_reader(&sf).unwrap(), sf),
            Err(_) => {
                std::fs::create_dir_all(&dir)
                    .unwrap_or_else(|_| todo!("Failed to create ~/.config/qst/ directory"));
                let sf = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(dir + "/appsearcher.yaml")
                    .unwrap_or_else(|_| {
                        todo!("Failed to create ~/.config/qst/appsearcher.yaml file")
                    });
                let f = File {
                    system_last_update: std::time::SystemTime::now(),
                    apps: std::collections::HashMap::new(),
                };
                serde_yaml::to_writer(&sf, &f).unwrap_or_else(|_| {
                    todo!("Failed to create default ~/.config/qst/appsearcher.yaml file")
                });
                (f, sf)
            }
        };
        if let Err(e) = std::fs::metadata("/usr/share/applications")
            .and_then(|m| m.modified())
            .and_then(|t| {
                if file.system_last_update != t {
                    file.system_last_update = t;
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "need update",
                    ))
                } else {
                    Ok(())
                }
            })
        {
            log::info(format!("update system: {}", e).as_str());
            update_system(&mut file.apps);
        };
        let mut id: u32 = 0;
        let mut trie = super::trie::Trie::new();
        let mut by_id = HashMap::new();
        file.apps.into_iter().for_each(|(name, attr)| {
            let app = Arc::new(App { name, id, attr });
            trie.insert(app.name.clone(), app.clone());
            by_id.insert(id, app);
            id += 1;
        });
        Self {
            file_last_update: sf.metadata().and_then(|m| m.modified()).ok(),
            system_last_update: file.system_last_update,
            by_id,
            trie,
        }
    }

    pub fn save(&self) {
        let dir = cfg_dir().expect("Failed to get config directory");
        let f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(dir + "/appsearcher.yaml")
            .unwrap_or_else(|_| todo!("Failed to create ~/.config/qst/appsearcher.yaml file"));
        serde_yaml::to_writer(
            &f,
            &File {
                system_last_update: self.system_last_update,
                apps: self
                    .by_id
                    .iter()
                    .map(|(_, app)| (app.name.clone(), app.attr.clone()))
                    .collect(),
            },
        )
        .unwrap_or_else(|_| todo!("Failed to create default ~/.config/qst/appsearcher.yaml file"));
    }
}
