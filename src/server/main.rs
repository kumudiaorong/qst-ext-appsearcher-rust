use super::file::{common, extension};
use crate::{
    config::{self, Appattr},
    trie::Trie,
};
use common::Empty;
use extension::ext_client::ExtClient;
use std::{collections::HashMap, sync::Mutex};
use tonic::Status;
use xlog::{debug, error, info};
#[derive(Debug)]
pub struct App {
    pub name: String,
    pub id: u32,
    pub attr: Appattr,
}
impl App {
    pub fn to_display(&self) -> extension::DisplayItem {
        extension::DisplayItem {
            obj_id: self.id,
            name: self.name.clone(),
            hint: None,
        }
    }
}
pub struct Main {
    id: String,
    addr: String,
    trie: Trie<u32>,
    by_id: HashMap<u32, App>,
    cli: Mutex<ExtClient<tonic::transport::Channel>>,
    config: Mutex<config::Config>,
    children: Mutex<HashMap<u32, Vec<std::process::Child>>>,
}

impl Main {
    pub async fn new(id: String, ep: tonic::transport::Endpoint, addr: String) -> Self {
        info!("server addr: {}", addr);
        let cfg = config::init();
        let ret = cfg.inner.apps.iter().enumerate().map(|(id, (name, attr))| {
            let id = id as u32;
            let app = App {
                name: name.clone(),
                id,
                attr: attr.clone(),
            };
            ((id, app), (name.clone(), id))
        });
        let (by_id, trie): (HashMap<_, _>, Trie<_>) = ret.unzip();
        let mut client;
        match ep.connect().await {
            Ok(c) => {
                debug!("connected to {}", ep.uri());
                client = ExtClient::new(c);
                client
                    .config_ext(extension::ExtConfigHint {
                        id: id.clone(),
                        addr: "http://".to_owned() + &addr,
                        expire: None,
                    })
                    .await
                    .unwrap();
            }
            Err(e) => {
                error!("failed to connect to {}: {}", ep.uri(), e);
                std::process::exit(1);
            }
        }
        Self {
            id,
            addr,
            trie,
            by_id,
            cli: Mutex::new(client),
            config: Mutex::new(config::Config::new()),
            children: Mutex::new(HashMap::new()),
        }
    }
}

#[tonic::async_trait]
impl extension::main_server::Main for Main {
    async fn search(
        &self,
        request: tonic::Request<extension::Input>,
    ) -> std::result::Result<tonic::Response<extension::DisplayList>, Status> {
        let inner = request.into_inner();
        let list = self
            .trie
            .search_prefix(inner.content)
            .iter()
            .map(|id| self.by_id.get(id).unwrap().to_display())
            .collect();
        Ok(tonic::Response::new(extension::DisplayList { list }))
    }
    async fn submit(
        &self,
        request: tonic::Request<extension::SubmitHint>,
    ) -> std::result::Result<tonic::Response<Empty>, Status> {
        let shint = request.into_inner();
        self.by_id
            .get(&shint.obj_id)
            .ok_or(Status::new(
                tonic::Code::NotFound,
                format!("id {} not found", shint.obj_id),
            ))
            .and_then(|app| {
                info!("executing {},dir: {}", app.attr.exec, app.attr.dir);
                std::process::Command::new(&app.attr.exec)
                    .current_dir(&app.attr.dir)
                    .spawn()
                    .map_err(|e| Status::new(tonic::Code::Unknown, format!("{}", e)))
            })
            .map(|child| {
                self.children
                    .lock()
                    .unwrap()
                    .entry(shint.obj_id)
                    .or_default()
                    .push(child)
            })
            .map(|_| tonic::Response::new(Empty {}))
    }
}
