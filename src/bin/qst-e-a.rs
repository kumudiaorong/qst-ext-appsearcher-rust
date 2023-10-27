use qst_ext_appsearcher_rust::{flag, server};
#[tokio::main]
async fn main() -> Result<(), tonic::transport::Error> {
    xlog_rs::log::init(std::io::stdout(), xlog_rs::log::Level::Trace);
    let flag = flag::Flags::new(std::env::args().collect());
    let tl = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = tl.local_addr().unwrap().to_string();
    tonic::transport::Server::builder()
        .add_service(server::MainServer::new(
            server::Main::new(flag.id, flag.endpoint, addr).await,
        ))
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(tl))
        .await
    // let mut cfg = qst_ext_appsearcher_rust::config::Config::new();
    // cfg.save();
    // todo!()
    // let mut cfg = config::Config::new();
    // cfg.file.exts.insert(
    //     0,
    //     config::Appattr {
    //         name: "qst-ext-appseracher-rust".to_string(),
    //         prompt: "".to_string(),
    //         dir: "/home/kmdr/pro/qst-ext-appsearcher-rust".to_string(),
    //         exec: "ls".to_string(),
    //         addr: "".to_string(),
    //     },
    // );
    // cfg.save();
    // println!("{:?}", cfg);
}
