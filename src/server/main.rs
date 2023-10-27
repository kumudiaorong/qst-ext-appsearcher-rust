use super::file::extension::DisplayList;
use super::file::{daemon, defs, extension};
use crate::config;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::Mutex;
use xlog_rs::log;
pub struct Main {
    id: String,
    addr: String,
    cli: Mutex<daemon::ext_client::ExtClient<tonic::transport::Channel>>,
    config: Mutex<config::Config>,
    children: Mutex<HashMap<u32, Vec<std::process::Child>>>,
}

impl Main {
    pub async fn new(id: String, ep: tonic::transport::Endpoint, addr: String) -> Self {
        log::info(format!("server addr: {}", addr).as_str());
        let mut client;
        match ep.connect().await {
            Ok(c) => {
                log::info(format!("connected to {}", ep.uri()).as_str());
                client = daemon::ext_client::ExtClient::new(c);
                client
                    .set_ext_addr(daemon::ExtAddrWithId {
                        id: id.clone(),
                        addr: addr.clone(),
                    })
                    .await
                    .unwrap();
            }
            Err(e) => {
                log::error(format!("failed to connect to {}: {}", ep.uri(), e).as_str());
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
    ) -> std::result::Result<tonic::Response<extension::DisplayList>, tonic::Status> {
        let inner = request.into_inner();
        log::info(format!("search: {}", inner.content).as_str());
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
    ) -> std::result::Result<tonic::Response<defs::Empty>, tonic::Status> {
        let shint = request.into_inner();
        self.config
            .lock()
            .unwrap()
            .by_id
            .get(&shint.obj_id)
            .ok_or(tonic::Status::new(
                tonic::Code::NotFound,
                format!("id {} not found", shint.obj_id),
            ))
            .and_then(|app| {
                log::info(format!("executing {},dir: {}", app.attr.exec, app.attr.dir).as_str());
                std::process::Command::new(&app.attr.exec)
                    .current_dir(&app.attr.dir)
                    .spawn()
                    .map_err(|e| tonic::Status::new(tonic::Code::Unknown, format!("{}", e)))
            })
            .map(|child| {
                self.children
                    .lock()
                    .unwrap()
                    .entry(shint.obj_id)
                    .or_default()
                    .push(child)
            })
            .map(|_| tonic::Response::new(defs::Empty {}))
    }
}
