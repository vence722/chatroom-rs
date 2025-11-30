use crate::types::ServerState;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

pub(crate) async fn broadcast_message(
    state: Arc<Mutex<ServerState>>,
    conn_id: Arc<str>,
    msg: Vec<u8>,
) -> anyhow::Result<()> {
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
