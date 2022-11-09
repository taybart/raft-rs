mod raft;
use raft::{protocol, rpc};

const ADDRESS: &'static str = "127.0.0.1:9090";

const SERVER: bool = false;

#[tokio::main]
async fn main() {
    if !SERVER {
        let c = rpc::Client {
            addr: ADDRESS.to_string(),
        };
        c.request_vote(protocol::VoteRequest {
            other_thing: "woot".to_string(),
            term: 1,
            candidate_id: 0,
            last_log_index: 0,
            last_log_term: 0,
        })
        .await
        .unwrap();
    } else {
        let s = rpc::Server {
            addr: ADDRESS.to_string(),
        };
        let _ = s.listen().await.map_err(|_| "woah");
    }
}
