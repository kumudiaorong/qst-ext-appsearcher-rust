use appsearcher::{arg, server};
use clap::Parser;
use tonic::transport;
use xlog::log;
#[tokio::main]
async fn main() -> Result<(), transport::Error> {
    log::init(std::io::stdout(), log::Level::Trace);
    let args = arg::Args::parse();
    let tl = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = tl.local_addr().unwrap().to_string();
    let signal = appsearcher::shutdown::init();
    transport::Server::builder()
        .add_service(server::MainServer::new(
            server::Main::new(
                args.id,
                transport::Channel::from_shared(args.uri).unwrap(),
                addr,
            )
            .await,
        ))
        .serve_with_incoming_shutdown(tokio_stream::wrappers::TcpListenerStream::new(tl), signal)
        .await?;
    Ok(())
}
