use std::collections::HashMap;
use std::sync::{atomic, Arc};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;

#[derive(Default)]
pub(crate) struct ServerState {
    pub(crate) connections: HashMap<Arc<str>, Arc<Mutex<OwnedWriteHalf>>>,
    pub(crate) connections_count: atomic::AtomicUsize,
}