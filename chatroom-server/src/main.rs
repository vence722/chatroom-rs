use std::collections::HashMap;
use std::sync::{atomic, Arc};
use std::time::Duration;
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;
use uuid::Uuid;

const SERVER_HOST: &str = "0.0.0.0";
const SERVER_PORT: u16 = 8892;

#[derive(Default)]
struct ServerState {
    connections: HashMap<Arc<str>, Arc<Mutex<OwnedWriteHalf>>>,
    connections_count: atomic::AtomicUsize
}

async fn report_connection_count(state: Arc<Mutex<ServerState>>) -> Result<()> {
    loop {
        let connection_count = state.lock().await.connections_count.load(atomic::Ordering::Relaxed);
        println!("Current connection count: {}", connection_count);
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn broadcast_message(state: Arc<Mutex<ServerState>>, conn_id: Arc<str>, msg: Vec<u8>) -> Result<()> {
    // let mut broadcast_count = 0;
    let mut connection_refs = Vec::with_capacity(128);
    {
        for (target_conn_id, target_stream) in state.lock().await.connections.iter() {
            if *target_conn_id == conn_id {
                continue;
            }
            connection_refs.push(Arc::clone(target_stream));
        }
    }
    let broadcast_count = connection_refs.len();
    for connection_ref in connection_refs.into_iter() {
        let mut conn = connection_ref.lock().await;
        conn.write_all(msg.as_ref()).await?;
        conn.flush().await?;
    }
    println!("Broadcast a message to {} clients", broadcast_count);
    Ok(())
}

async fn register_connection(state: &Arc<Mutex<ServerState>>, conn_id: &Arc<str>, write_stream: OwnedWriteHalf) -> Result<()> {
    let mut state_mut = state.lock().await;
    state_mut.connections.insert(Arc::clone(&conn_id), Arc::new(Mutex::new(write_stream)));
    state_mut.connections_count.fetch_add(1, atomic::Ordering::Relaxed);
    println!("Connection established: {}", conn_id);
    Ok(())
}

async fn unregister_connection(state: &Arc<Mutex<ServerState>>, conn_id: &Arc<str>) -> Result<()> {
    let mut state_mut = state.lock().await;
    state_mut.connections.remove(conn_id);
    state_mut.connections_count.fetch_sub(1, atomic::Ordering::Relaxed);
    println!("Connection closed: {}", conn_id);
    Ok(())
}

async fn handle_connection(state: Arc<Mutex<ServerState>>, stream: TcpStream) -> Result<()> {
    let conn_id: Arc<str> = Uuid::new_v4().to_string().into();
    let (read_stream, write_stream) = stream.into_split();
    // Register connection
    register_connection(&state, &conn_id, write_stream).await?;
    let mut reader = BufReader::new(read_stream);
    let mut buf = String::with_capacity(1024);
    while let Ok(bytes_read) = reader.read_line(&mut buf).await {
        if bytes_read == 0 { break; }
        // Broadcast message to all connections except self
        let buf_cloned = buf.as_bytes().to_vec();
        tokio::spawn(broadcast_message(Arc::clone(&state), Arc::clone(&conn_id), buf_cloned));
        buf.clear();
    }
    // Unregister connection
    unregister_connection(&state, &conn_id).await?;
    Ok(())
}

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
