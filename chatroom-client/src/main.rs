use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf};
use tokio::net::TcpStream;

const SERVER_ADDRESS: &str = "0.0.0.0:8892";

async fn handle_receive_message(read_stream: OwnedReadHalf) -> Result<()> {
    let mut reader = BufReader::new(read_stream);
    let mut buf = String::with_capacity(1024);
    while let Ok(_) = reader.read_line(&mut buf).await {
        println!("receive message: {}", buf.trim());
        buf.clear();
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()>{
    let (read_stream, mut write_stream) = TcpStream::connect(SERVER_ADDRESS).await?.into_split();
    tokio::spawn(handle_receive_message(read_stream));
    let msg = "Hello, world!\n";
    loop {
        write_stream.write_all(msg.as_bytes()).await?;
        write_stream.flush().await?;
        println!("sent message: {}", msg.trim());
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
