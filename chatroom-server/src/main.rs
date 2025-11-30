mod constant;
mod handler;
mod task;
mod types;

use crate::constant::{SERVER_HOST, SERVER_PORT};
use crate::handler::handle_connection;
use crate::task::report_connection_count;

use crate::types::ServerState;
use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialization
    let listener = TcpListener::bind(&format!("{}:{}", SERVER_HOST, SERVER_PORT)).await?;
    let state = Arc::new(Mutex::new(ServerState::default()));
    // Spawn report connection count task
    tokio::spawn(report_connection_count(Arc::clone(&state)));
    // Handling incoming connections
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(Arc::clone(&state), stream));
    }
    Ok(())
}
