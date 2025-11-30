use crate::task::broadcast_message;
use crate::types::ServerState;
use std::sync::{Arc, atomic};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;
use uuid::Uuid;

pub(crate) async fn handle_connection(
    state: Arc<Mutex<ServerState>>,
    stream: TcpStream,
) -> anyhow::Result<()> {
    let conn_id: Arc<str> = Uuid::new_v4().to_string().into();
    let (read_stream, write_stream) = stream.into_split();
    // Register connection
    register_connection(&state, &conn_id, write_stream).await?;
    let mut reader = BufReader::new(read_stream);
    let mut buf = String::with_capacity(1024);
    while let Ok(bytes_read) = reader.read_line(&mut buf).await {
        if bytes_read == 0 {
            break;
        }
        // Broadcast message to all connections except self
        let buf_cloned = buf.as_bytes().to_vec();
        tokio::spawn(broadcast_message(
            Arc::clone(&state),
            Arc::clone(&conn_id),
            buf_cloned,
        ));
        buf.clear();
    }
    // Unregister connection
    unregister_connection(&state, &conn_id).await?;
    Ok(())
}

async fn register_connection(
    state: &Arc<Mutex<ServerState>>,
    conn_id: &Arc<str>,
    write_stream: OwnedWriteHalf,
) -> anyhow::Result<()> {
    let mut state_mut = state.lock().await;
    state_mut
        .connections
        .insert(Arc::clone(&conn_id), Arc::new(Mutex::new(write_stream)));
    state_mut
        .connections_count
        .fetch_add(1, atomic::Ordering::Relaxed);
    println!("Connection established: {}", conn_id);
    Ok(())
}

async fn unregister_connection(
    state: &Arc<Mutex<ServerState>>,
    conn_id: &Arc<str>,
) -> anyhow::Result<()> {
    let mut state_mut = state.lock().await;
    state_mut.connections.remove(conn_id);
    state_mut
        .connections_count
        .fetch_sub(1, atomic::Ordering::Relaxed);
    println!("Connection closed: {}", conn_id);
    Ok(())
}
