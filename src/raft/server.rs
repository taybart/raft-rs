use super::protocol::{AppendRequest, AppendResult, LogEntry, VoteRequest, VoteResponse};
use super::protocol::{Function, Rpc};
use prost::{bytes::Bytes, Message};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::{
    net::{TcpListener, TcpStream},
    time::{sleep, Duration},
};

const ELECTION_TIMEOUT_MS: u128 = 3_000;

#[derive(Default)]
pub struct Server {
    pub addr: String,
}

#[derive(Default, PartialEq)]
pub enum Role {
    #[default]
    Follower,
    Candidate,
    Leader,
}

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
    role: Role,
    current_term: u64, // monotonic
    _voted_for: String,
    log: Vec<LogEntry>,

    last_leader_contact: WrapInstant,

    /* Voliatile */
    // all servers
    commit_index: u64,
    last_applied: u64,
    // leaders (reinitialized after election)
    _next_index: Vec<u64>,
    _match_index: Vec<u64>,

    friends: Vec<String>,
}

impl Server {
    pub fn handle_append_entries(state: SState, mut req: AppendRequest) -> Vec<u8> {
        let mut state = state.lock().unwrap();

        let mut ret = AppendResult {
            term: state.current_term,
            success: true,
        };

        // heartbeat
        if req.entries.len() == 0 {
            println!("heartbeat");
            return ret.encode_to_vec();
        }

        // they are behind
        if req.term < state.current_term {
            ret.success = false;
            return ret.encode_to_vec();
        }

        // if req.leader_id != state.

        // we are behind
        if req.term > state.current_term {
            state.role = Role::Follower;
            state.current_term = req.term;
        }

        // jumping the line?
        // let entry = state.log.get(state.current_term);

        state.log.append(&mut req.entries);
        state.commit_index += 1;

        println!(
            "updated log idx({}) {:?} from {}",
            state.commit_index,
            state.log.len(),
            req.id,
        );

        state.last_leader_contact = WrapInstant(Instant::now());
        ret.encode_to_vec()
    }
    pub fn handle_vote_request(state: SState, req: VoteRequest) -> Vec<u8> {
        println!("Vote request in term {}!", req.term);

        let mut ret = VoteResponse {
            term: req.term,
            vote_grated: true,
        };
        let mut state = state.lock().unwrap();

        if req.term < state.current_term {
            ret.vote_grated = false;
            return ret.encode_to_vec();
        }

        // we are behind
        if req.term > state.current_term {
            state.role = Role::Follower;
            state.current_term = req.term;
        }

        /*
         * If votedFor is null or candidateId, and candidate’s log is at
         * least as up-to-date as receiver’s log, grant vote
         */
        if req.candidate_id == state.id {
            println!("someone voted for me!");
            state.role = Role::Leader;
        }

        ret.encode_to_vec()
    }

    async fn process(state: SState, stream: TcpStream) -> Result<(), String> {
        let now = Instant::now();

        stream
            .readable()
            .await
            .map_err(|_| "could not check if stream is readable")?;

        let mut buf = vec![0; 4096];
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
                Self::handle_vote_request(state, req)
            }
            Some(Function::AppendEntries) => {
                let req = AppendRequest::decode(Bytes::copy_from_slice(&rpc_req.request)).unwrap();
                Self::handle_append_entries(state, req)
            }
            None => {
                panic!("unknown rpc")
            }
        };

        // we already checked (+panicked) if call is known above
        let func = Function::from_i32(rpc_req.call)
            .expect("unknown rpc")
            .as_str_name();
        stream.try_write(&res).expect("writing response failed");
        println!("{} done in {}μs", func, now.elapsed().as_micros());
        Ok(())
    }
    async fn follower(state: SState) {
        loop {
            // 20ms tick
            sleep(Duration::from_millis(20)).await;
            let mut s = state.lock().unwrap();
            if s.last_leader_contact.0.elapsed().as_millis() > ELECTION_TIMEOUT_MS
                && s.role != Role::Leader
            {
                s.role = Role::Candidate;
                s.current_term += 1;
                println!(
                    "no contact from leader, requesting election for term {}",
                    s.current_term
                );

                let req = VoteRequest {
                    term: s.current_term,
                    candidate_id: s.id,
                    last_log_index: s.last_applied,
                    last_log_term: s.current_term - 1,
                };
                println!("lets elect a new leader...why not me?");
                for friend in &s.friends {
                    println!("hello friend:{:?} my name is {}\n{:?}", friend, s.id, req);
                }
                // assuming that went well
                s.last_leader_contact.0 = Instant::now();
                // elected!
                // return;
            }
        }
    }

    pub async fn listen(self, friends: Vec<String>, role: Role) -> Result<(), String> {
        let listener = TcpListener::bind(self.addr.clone())
            .await
            .expect("failed to bind address");

        println!("listening on {}...", self.addr);

        // init state
        let mut s = State::default();
        s.role = role;
        s.friends = friends;
        let state = Arc::new(Mutex::new(s));
        {
            // internal timer
            let state = state.clone();
            tokio::spawn(async move {
                // loop to change state?
                Self::follower(state).await
            });
        }
        loop {
            let (socket, _) = listener
                .accept()
                .await
                .map_err(|_| "failed to accept incoming connection")?;

            let state = state.clone();
            tokio::spawn(async move {
                match Self::process(state, socket).await {
                    Ok(_) => {}
                    Err(e) => eprintln!("{}", e),
                };
            });
        }
    }
}
