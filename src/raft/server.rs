use super::protocol::{AppendRequest, AppendResult, LogEntry, VoteRequest, VoteResponse};
use super::protocol::{Function, RaftErr, Rpc};
use super::roles::Role::{Candidate, Follower, Leader};
use super::FRAME_SIZE;

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

    friends: Vec<crate::raft::protocol::Server>,
}
/*
 * handle_append_entries: follows raft convention for heartbeat/out-of-sync
 *                        also appends log entries to the log.
 */
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

    // reset election counter
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

/*
 * handle_vote_request: accepts vote requests incase a leader dies or newbies show up
 */
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

/*
 * process: take rpc and translate/run it
 */
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
        Some(Function::AddFriend) => {
            let req =
                crate::raft::protocol::Server::decode(Bytes::copy_from_slice(&rpc_req.request))
                    .unwrap();
            if let Ok(mut s) = state.lock() {
                for friend in s.friends.clone() {
                    // TODO: check addr also?
                    if friend.id != req.id {
                        println!("wow! new friend in the network! {:?}", req);
                        s.friends.push(req);
                        break;
                    }
                }
            }
            AppendResult {
                success: true,
                ..Default::default()
            }
            .encode_to_vec()
        }
        // should only handle leader => []follower
        Some(Function::NetworkDiscovery) => Vec::new(), //handle_discover(state),
        None => {
            stream
                .try_write(
                    &RaftErr {
                        msg: format!("wtf kind of rpc is that?").to_string(),
                    }
                    .encode_to_vec(),
                )
                .expect("writing response failed");
            return Ok(());
        }
    };

    stream.try_write(&res).expect("writing response failed");
    Ok(())
}

/*
 * tick: server janitor service
 */
async fn tick(state: SState) -> bool {
    let mut since_heartbeat = Instant::now();
    loop {
        // 20ms tick, this just keeps the server from red lining
        sleep(Duration::from_millis(20)).await;
        // keep mutex free as much as possible
        let mut should_vote = false;
        let mut should_heartbeat = false;
        if let Ok(mut s) = state.lock() {
            // no leader! this probably means they are dead
            // goto: election state, promote to candidate, increment term
            if s.last_leader_contact.0.elapsed().as_millis() > s.election_timeout_ms
                && s.role != Leader
            {
                s.role = Candidate;
                s.current_term += 1;
                println!(
                    "no contact from leader, requesting election for term {}",
                    s.current_term
                );
                // lets pick a new leader
                should_vote = true;
            }
            // if we are the leader we need to talk to our loyal subjects
            // so they know who is boss
            should_heartbeat =
                since_heartbeat.elapsed().as_millis() > HEARTBEAT_TIMEOUT_MS && s.role == Leader;
        }
        if should_heartbeat {
            // blast everyone with a nop append_entries
            heartbeat(state.clone()).await;
            // reset arbitrary timer (save on network bw)
            since_heartbeat = Instant::now();
        }
        if should_vote {
            // lets elect the most qualified (me), selfish election
            request_vote(state.clone()).await;
        }
    }
}

/*
 * heartbeat: hits everyone in state.friends with a nop
 * (append_entries with no logs)
 */
async fn heartbeat(state: SState) {
    let mut req = AppendRequest::default();
    let mut friends = Vec::new();
    // grab snapshot of state
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

    // let everyone know i am still in charge
    let tasks: Vec<_> = friends
        .iter_mut()
        .map(|friend| {
            tokio::spawn(crate::raft::client::append_entries(
                friend.addr.to_string(),
                req.clone(),
            ))
        })
        .collect();
    // wait for the decree to be done
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

/*
 * request_vote: there is no leader heartbeat
 * lets pick "us" as the new jefe
 */
async fn request_vote(state: SState) {
    let mut req = VoteRequest::default();
    let mut friends = Vec::new();
    // lock up our state so we can reset
    if let Ok(mut s) = state.lock() {
        // set voted for so zero value
        s.voted_for = 0;
        req = VoteRequest {
            term: s.current_term,
            candidate_id: s.id,
            last_log_index: s.last_applied,
            // this is meaningless
            last_log_term: if s.current_term == 0 {
                0
            } else {
                s.current_term - 1
            },
        };
        friends = s.friends.clone();
    }

    // have an election
    let tasks: Vec<_> = friends
        .iter_mut()
        .map(|friend| {
            tokio::spawn(crate::raft::client::request_vote(
                friend.addr.to_string(),
                req.clone(),
            ))
        })
        .collect();

    let mut yes_votes = 0;
    let mut responses = 0;
    for task in tasks {
        // TODO: throw out non responders possibly
        if let Ok(res) = task.await.unwrap() {
            // we got a response
            responses += 1;
            if res.vote_granted {
                // they think we should be in charge
                yes_votes += 1;
            }
        }
    }
    let mut election_won = false;
    if let Ok(mut s) = state.lock() {
        // make sure we got a majority vote
        if yes_votes as f64 > responses as f64 / 2.0 || responses == 0 {
            println!("i am the leader for term {} woohoo!", s.current_term);
            s.role = Leader;
            election_won = true;
        }
        s.last_leader_contact.0 = Instant::now();
    }

    // if we win immediately tell the flock
    if election_won {
        heartbeat(state.clone()).await;
    }
}

pub async fn listen(disco: String, addr: String, id: u64) -> Result<(), String> {
    // grab the interface
    let listener = TcpListener::bind(addr.clone())
        .await
        .expect("failed to bind address");

    print!("doing network discovery...");
    // TODO: should resort to normal self promotion on failure
    let disco_baby = crate::raft::client::network_discovery(
        disco.clone(),
        crate::raft::protocol::Server {
            id,
            addr: addr.clone(),
        },
    )
    .await
    .unwrap();

    // get lazy thread random before we split
    let mut rng = rand::thread_rng();
    // init state, election timeout is randomized to help prevent stalemates
    let s = State {
        id,
        election_timeout_ms: rng.gen_range(150..300),
        friends: disco_baby.friends.clone(),
        ..Default::default()
    };

    // create multi accessible shared state
    let state = Arc::new(Mutex::new(s));
    {
        let state = state.clone();
        tokio::spawn(async move {
            // do "server" janitor work
            // election timeout/leader jobs
            tick(state).await
        });
    }
    println!("listening on {}...", addr);

    // regular request processing
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
