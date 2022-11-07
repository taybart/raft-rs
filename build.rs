use std::io::Result;
// extern crate capnpc;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
fn main() -> Result<()> {
    // Ok(capnpc::CompilerCommand::new()
    //     .file("proto/raft.capnp")
    //     .run()?)

    prost_build::compile_protos(&["src/proto/raft.proto"], &["src/"])?;
    Ok(())
}
