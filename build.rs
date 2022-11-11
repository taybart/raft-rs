fn main() {
    prost_build::compile_protos(&["src/proto/raft.proto"], &["src/"]).expect("Protobuf compliation")
}
