use std::{
    future::Future,
    pin::Pin,
    sync::{atomic::AtomicBool, Arc},
};

use tokio::signal::unix::{signal, SignalKind};
pub fn init() -> Pin<Box<dyn Future<Output = ()>>> {
    let term = Arc::new(AtomicBool::new(false));
    for sig in signal_hook::consts::TERM_SIGNALS {
        signal_hook::flag::register(*sig, Arc::clone(&term)).unwrap();
    }
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigquit = signal(SignalKind::quit()).unwrap();
    Box::pin(async move {
        tokio::select! {
            _ = sigterm.recv() => {
            }
            _ = sigint.recv() => {
            }
            _ = sigquit.recv() => {
            }
        }
    })
}
