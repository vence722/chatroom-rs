fn main() -> std::io::Result<()> {
    prost_build::Config::new()
        .out_dir("src/pb")
        .compile_protos(&["proto/chatroom.proto"], &["proto/"])?;
    Ok(())
}