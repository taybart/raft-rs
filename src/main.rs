use prost::Message;

/*** RAFT ***/
pub mod raft {
    include!(concat!(env!("OUT_DIR"), "/raft.rs"));
}

// const ADDRESS: &'static str = "127.0.0.1:9090";

fn request_vote(req: raft::VoteRequest) -> Vec<u8> {
    let mut res = raft::VoteResponse::default();
    res.term = req.term;
    let mut buf = Vec::new();
    res.encode(&mut buf).unwrap();
    buf
}

// #[tokio::main]
fn main() {
    let req = raft::VoteRequest {
        term: 1,
        candidate_id: 0,
        last_log_index: 0,
        last_log_term: 0,
    };
    let enc_res = request_vote(req);

    let res = raft::VoteResponse::decode(enc_res.as_slice()).unwrap();

    println!("{}", res.term);
}
