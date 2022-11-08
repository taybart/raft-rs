pub mod protocol {
    include!(concat!(env!("OUT_DIR"), "/raft.rs"));
}

pub mod rpc;
