pub mod protocol {
    include!(concat!(env!("OUT_DIR"), "/raft.rs"));
}

pub mod client;
pub mod roles;
pub mod server;

pub mod discovery;

pub const FRAME_SIZE: usize = 4_096;
