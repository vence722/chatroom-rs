use crate::types::ServerState;
use std::sync::{Arc, atomic};
use std::time::Duration;
use tokio::sync::Mutex;

pub(crate) async fn report_connection_count(state: Arc<Mutex<ServerState>>) -> anyhow::Result<()> {
    loop {
        let connection_count = state
            .lock()
            .await
            .connections_count
            .load(atomic::Ordering::Relaxed);
        println!("Current connection count: {}", connection_count);
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
