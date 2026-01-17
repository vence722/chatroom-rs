use std::sync::{atomic, Arc};
use dashmap::DashMap;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;

#[derive(Default)]
pub(crate) struct ServerState {
    pub(crate) connections: DashMap<Arc<str>, Arc<Mutex<OwnedWriteHalf>>>,
    pub(crate) connections_count: atomic::AtomicUsize,
}