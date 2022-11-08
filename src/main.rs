mod raft;
use raft::{protocol, rpc};

// const ADDRESS: &'static str = "127.0.0.1:9090";

#[tokio::main]
async fn main() {
    rpc::request_vote(protocol::VoteRequest {
        term: 1,
        candidate_id: 0,
        last_log_index: 0,
        last_log_term: 0,
    })
    .await
    .unwrap();
}
