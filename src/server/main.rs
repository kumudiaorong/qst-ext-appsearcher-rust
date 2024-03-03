use super::file::{common, extension};
use crate::config;
use common::Empty;
use extension::ext_client::ExtClient;
use std::{collections::HashMap, sync::Mutex};
use tonic::Status;
use xlog::{debug, error, info};
pub struct Main {
    id: String,
    addr: String,
    cli: Mutex<ExtClient<tonic::transport::Channel>>,
    config: Mutex<config::Config>,
    children: Mutex<HashMap<u32, Vec<std::process::Child>>>,
}

impl Main {
    pub async fn new(id: String, ep: tonic::transport::Endpoint, addr: String) -> Self {
        info!("server addr: {}", addr);
        let mut client;
        match ep.connect().await {
            Ok(c) => {
                debug!("connected to {}", ep.uri());
                client = ExtClient::new(c);
                client
                    .set_ext_addr(extension::ExtAddrWithId {
                        id: id.clone(),
                        addr: "http://".to_owned() + &addr,
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
        info!("search: {}", inner.content);
        Ok(tonic::Response::new(extension::DisplayList {
            list: self
                .config
                .lock()
                .unwrap()
                .trie
                .search_prefix(inner.content)
                .iter()
                .map(|app| extension::DisplayItem {
                    obj_id: app.id,
                    name: app.name.clone(),
                    hint: None,
                })
                .collect(),
        }))
    }
    async fn submit(
        &self,
        request: tonic::Request<extension::SubmitHint>,
    ) -> std::result::Result<tonic::Response<Empty>, Status> {
        let shint = request.into_inner();
        self.config
            .lock()
            .unwrap()
            .by_id
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
