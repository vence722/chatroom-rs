use crate::types::ServerState;
use std::sync::{Arc, atomic};
use std::time::Duration;

pub(crate) async fn report_connection_count(state: Arc<ServerState>) -> anyhow::Result<()> {
    loop {
        let connection_count = state
            .connections_count
            .load(atomic::Ordering::Relaxed);
        println!("Current connection count: {}", connection_count);
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
