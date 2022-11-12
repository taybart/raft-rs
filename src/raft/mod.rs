pub mod protocol {
    include!(concat!(env!("OUT_DIR"), "/raft.rs"));
}

pub mod client;
pub mod network;
pub mod roles;
pub mod server;
pub mod server_test;
