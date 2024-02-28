#[cfg(target_os = "linux")]
mod inner {
    use crate::config::Appattr;
    use std::collections::HashMap;
    use xlog_rs::warn;
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
    pub fn update_system(src: &mut HashMap<String, Appattr>) {
        match std::fs::read_dir("/usr/share/applications") {
            Ok(rd) => {
                for entry in rd {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_dir() {
                        continue;
                    }
                    let attr = std::fs::read_to_string(path.clone()).map_or_else(
                        |e| {
                            warn!("can not read {}: {}", path.to_str().unwrap(), e,);
                            None
                        },
                        |content: String| extract_app(&content),
                    );
                    if let Some((name, app)) = attr {
                        src.entry(name)
                            .and_modify(|e| {
                                *e = app.clone();
                            })
                            .or_insert(app.clone());
                    } else {
                        warn!("can not extract {}", path.to_str().unwrap());
                    }
                }
            }
            Err(_) => {
                warn!("can not read /usr/share/applications");
            }
        };
    }
}

pub use inner::update_system;
