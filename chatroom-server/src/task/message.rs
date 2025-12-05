use crate::types::ServerState;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

pub(crate) async fn broadcast_message(
    state: Arc<ServerState>,
    conn_id: Arc<str>,
    msg: Vec<u8>,
) -> anyhow::Result<()> {
    // let mut broadcast_count = 0;
    let mut connection_refs = Vec::with_capacity(128);
    {
        for connection_entry in state.connections.iter() {
            if *connection_entry.key() == conn_id {
                continue;
            }
            connection_refs.push(Arc::clone(connection_entry.value()));
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
