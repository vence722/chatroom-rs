use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use uuid::Uuid;

const SERVER_PORT: u16 = 8892;

#[derive(Default)]
struct ServerState {
    connections: HashMap<Arc<str>, Arc<Mutex<BufStream<TcpStream>>>>
}

async fn broadcast_message(state: Arc<Mutex<ServerState>>, conn_id: Arc<str>, msg: Vec<u8>) -> Result<()> {
    let mut broadcast_count = 0;
    for (target_conn_id, target_stream) in state.lock().await.connections.iter() {
        if *target_conn_id == conn_id {
            break;
        }
        let mut target_stream = target_stream.lock().await;
        target_stream.write_all(&msg).await?;
        broadcast_count += 1;
    }
    println!("Broadcast a message to {} clients", broadcast_count);
    Ok(())
}

async fn handle_connection(state: Arc<Mutex<ServerState>>, stream: TcpStream) -> Result<()> {
    let conn_id: Arc<str> = Uuid::new_v4().to_string().into();
    let stream = Arc::new(Mutex::new(BufStream::new(stream)));
    // Register connection
    {
        state.lock().await.connections.insert(Arc::clone(&conn_id), Arc::clone(&stream));
        println!("Connection established: {}", conn_id);
    }
    let mut stream_mut = stream.lock().await;
    let mut buf = String::new();
    while let Ok(bytes_read) = stream_mut.read_line(&mut buf).await {
        if bytes_read == 0 { break; }
        // Broadcast message to all connections except self
        let buf_cloned = buf.as_bytes().to_vec();
        tokio::spawn(broadcast_message(Arc::clone(&state), Arc::clone(&conn_id), buf_cloned));
        buf.clear();
    }
    // Unregister connection
    {
        state.lock().await.connections.remove(&conn_id);
        println!("Connection closed: {}", conn_id);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind(&format!("0.0.0.0:{}", SERVER_PORT)).await?;
    let state = Arc::new(Mutex::new(ServerState::default()));
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(Arc::clone(&state), stream));
    }
    Ok(())
}
