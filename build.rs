extern crate capnpc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(capnpc::CompilerCommand::new()
        .file("proto/raft.capnp")
        .run()?)
}
