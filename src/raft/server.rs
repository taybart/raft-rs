use super::network::FRAME_SIZE;
use super::protocol::{AppendRequest, AppendResult, LogEntry, VoteRequest, VoteResponse};
use super::protocol::{Function, Rpc};
use super::roles::Role::{Candidate, Follower, Leader};

use prost::{bytes::Bytes, Message};
use rand::prelude::*;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::{
    net::{TcpListener, TcpStream},
    time::{sleep, Duration},
};

// should be randomly selected 150-300ms
// const ELECTION_TIMEOUT_MS: u128 = 3_000;
const HEARTBEAT_TIMEOUT_MS: u128 = 50;

struct WrapInstant(std::time::Instant);

impl Default for WrapInstant {
    fn default() -> Self {
        WrapInstant(Instant::now())
    }
}

type SState = Arc<Mutex<State>>;
#[derive(Default)]
pub struct State {
    /* internal */
    id: u64,
    role: super::roles::Role,
    current_term: u64, // monotonic
    voted_for: u64,
    log: Vec<LogEntry>,

    last_leader_contact: WrapInstant,
    // now dynamic?
    election_timeout_ms: u128,

    /* Voliatile */
    // all servers
    commit_index: u64,
    last_applied: u64,
    // leaders (reinitialized after election)
    _next_index: Vec<u64>,
    _match_index: Vec<u64>,

    friends: Vec<&'static str>,
}
pub fn handle_append_entries(state: SState, mut req: AppendRequest) -> Vec<u8> {
    let mut state = state.lock().unwrap();

    let mut ret = AppendResult {
        term: state.current_term,
        success: true,
    };

    // they are behind
    if req.term < state.current_term {
        ret.success = false;
        return ret.encode_to_vec();
    // we are behind
    } else if req.term > state.current_term {
        state.role = Follower;
        state.current_term = req.term;
    }

    state.last_leader_contact.0 = Instant::now();

    // heartbeat
    if req.entries.len() == 0 {
        return ret.encode_to_vec();
    }

    state.log.append(&mut req.entries);
    state.commit_index += 1;

    println!(
        "updated log idx({}) {:?} from {}",
        state.commit_index,
        state.log.len(),
        req.id,
    );

    ret.encode_to_vec()
}
pub fn handle_vote_request(state: SState, req: VoteRequest) -> Vec<u8> {
    println!(
        "vote request in term {} for {}!",
        req.term, req.candidate_id
    );

    let mut ret = VoteResponse {
        term: req.term,
        vote_granted: true,
    };
    if let Ok(mut s) = state.lock() {
        if req.term < s.current_term {
            ret.vote_granted = false;
            println!("requested term is old");
            return ret.encode_to_vec();
        }
        // if s.role == Leader {
        //     ret.vote_granted = false;
        //     println!("i am the captain of this term");
        //     return ret.encode_to_vec();
        // }

        // we are behind, reset
        if req.term > s.current_term {
            s.role = Follower;
            s.current_term = req.term;
            s.voted_for = 0;
        }

        ret.vote_granted = s.voted_for == 0;
        if ret.vote_granted {
            s.voted_for = req.candidate_id;
            println!(
                "vote request response voted for {} in term {}",
                s.voted_for, s.current_term
            );
        };
    }
    ret.encode_to_vec()
}

async fn process(state: SState, stream: TcpStream) -> Result<(), String> {
    stream
        .readable()
        .await
        .map_err(|e| format!("stream not readable {}", e))?;

    let mut buf = vec![0; FRAME_SIZE];
    let _n = match stream.try_read(&mut buf) {
        Ok(n) => buf.truncate(n),
        Err(e) => {
            eprintln!("failed to read from socket; err = {:?}", e)
        }
    };
    let rpc_req = Rpc::decode(Bytes::copy_from_slice(&buf)).unwrap();
    let res = match Function::from_i32(rpc_req.call) {
        Some(Function::RequestVote) => {
            let req = VoteRequest::decode(Bytes::copy_from_slice(&rpc_req.request)).unwrap();
            handle_vote_request(state, req)
        }
        Some(Function::AppendEntries) => {
            let req = AppendRequest::decode(Bytes::copy_from_slice(&rpc_req.request)).unwrap();
            handle_append_entries(state, req)
        }
        None => {
            panic!("unknown rpc")
        }
    };

    stream.try_write(&res).expect("writing response failed");
    Ok(())
}

async fn tick(state: SState) -> bool {
    let mut since_heartbeat = Instant::now();
    loop {
        // 20ms tick
        sleep(Duration::from_millis(20)).await;
        let mut should_vote = false;
        let mut should_heartbeat = false;
        if let Ok(mut s) = state.lock() {
            // no leader!
            if s.last_leader_contact.0.elapsed().as_millis() > s.election_timeout_ms
                && s.role != Leader
            {
                s.role = Candidate;
                s.current_term += 1;
                println!(
                    "no contact from leader, requesting election for term {}",
                    s.current_term
                );
                should_vote = true;
            }
            should_heartbeat =
                since_heartbeat.elapsed().as_millis() > HEARTBEAT_TIMEOUT_MS && s.role == Leader;
        }
        if should_heartbeat {
            heartbeat(state.clone()).await;
            since_heartbeat = Instant::now();
        }
        if should_vote {
            request_vote(state.clone()).await;
        }
    }
}

async fn heartbeat(state: SState) {
    let mut req = AppendRequest::default();
    let mut friends = Vec::new();
    if let Ok(s) = state.lock() {
        req = AppendRequest {
            id: s.id,
            term: s.current_term,
            leader_id: s.id,
            entries: vec![],
            prev_log_index: s.last_applied,
            prev_log_term: s.current_term,
        };
        friends = s.friends.clone();
    }

    let tasks: Vec<_> = friends
        .iter_mut()
        .map(|friend| {
            tokio::spawn(crate::raft::client::append_entries(
                friend.to_string(),
                req.clone(),
            ))
        })
        .collect();
    for task in tasks {
        match task.await.unwrap() {
            Ok(res) => {
                if !res.success {
                    if let Ok(mut s) = state.lock() {
                        println!("hb failed term {} {:?}", s.current_term, res);
                        if s.current_term < res.term {
                            s.current_term = res.term;
                        }
                    }
                }
            }
            Err(_) => {
                // eprintln!("{}", e);
            }
        }
    }
}
async fn request_vote(state: SState) {
    let mut req = VoteRequest::default();
    let mut friends = Vec::new();
    if let Ok(mut s) = state.lock() {
        s.voted_for = 0;
        req = VoteRequest {
            term: s.current_term,
            candidate_id: s.id,
            last_log_index: s.last_applied,
            last_log_term: if s.current_term == 0 {
                0
            } else {
                s.current_term - 1
            },
        };
        friends = s.friends.clone();
    }

    let tasks: Vec<_> = friends
        .iter_mut()
        .map(|friend| {
            tokio::spawn(crate::raft::client::request_vote(
                friend.to_string(),
                req.clone(),
            ))
        })
        .collect();

    let mut yes_votes = 0;
    let mut responses = 0;
    for task in tasks {
        if let Ok(res) = task.await.unwrap() {
            responses += 1;
            if res.vote_granted {
                yes_votes += 1;
            }
        }
    }
    let mut election_won = false;
    if let Ok(mut s) = state.lock() {
        if yes_votes as f64 > responses as f64 / 2.0 || responses == 0 {
            println!("i am the leader for term {} woohoo!", s.current_term);
            s.role = Leader;
            election_won = true;
        }
        s.last_leader_contact.0 = Instant::now();
    }

    if election_won {
        heartbeat(state.clone()).await;
    }
}

pub async fn listen(
    addr: String,
    id: u64,
    friends: Vec<&'static str>,
    role: super::roles::Role,
) -> Result<(), String> {
    let listener = TcpListener::bind(addr.clone())
        .await
        .expect("failed to bind address");

    println!("listening on {}...", addr);

    let mut rng = rand::thread_rng();

    // init state
    let s = State {
        id,
        role,
        friends,
        election_timeout_ms: rng.gen_range(150..300),
        ..Default::default()
    };

    let state = Arc::new(Mutex::new(s));
    {
        // internal timer
        let state = state.clone();
        tokio::spawn(async move {
            // loop to change state?
            tick(state).await
        });
    }
    loop {
        let (socket, _) = listener
            .accept()
            .await
            .map_err(|_| "failed to accept incoming connection")?;

        let state = state.clone();
        tokio::spawn(async move {
            process(state, socket).await.unwrap();
        });
    }
}
