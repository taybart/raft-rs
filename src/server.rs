// If a follower receives no communication over a period of time called the election timeout,
// then it assumes there is no viable leader and begins an election to choose a new leader
pub const ELECTION_TIMEOUT: u32 = 100; // ms

pub struct State {
    // Persistent (updated on stable storage before responding to RPCs)
    role: Role,
    current_term: u64, // monotonic
    voted_for: u64,
    log: Vec<String>,

    // Voliatile (all servers)
    commit_index: u64,
    last_applied_index: u64,

    // Voliatile (leaders) (reinitialized after election)
    next_index: Vec<u64>,
    match_index: Vec<u64>,
}

/* ******** SERVERS ********
 * When servers start up, they begin as followers.
 * A server remains in follower state as long as it
 * receives valid 5 RPCs from a leader or candidate.
 */

/*
 * where is the "entry fee" here?
 */

pub enum Role {
    Follower,
    Candidate,
    Leader,
}

pub struct Server {
    id: u64,
    guest_list: Vec<String>,
    state: State,
    // id: u64,
    // role: Role,
    // // Persistent (updated on stable storage before responding to RPCs)
    // current_term: u64, // monotonic
    // // voted_for: String,
    // voted_for: u64,
    // log: Vec<String>,

    // // Voliatile (all servers)
    // commit_index: u64,
    // last_applied_index: u64,

    // // Voliatile (leaders) (reinitialized after election)
    // next_index: Vec<u64>,
    // match_index: Vec<u64>,
}

/*** RPCs ***/

impl Server {
    pub fn new() -> Server {
        Server {
            id: 0,
            guest_list: Vec::new(),
            state: State{
                role: Role::Follower,
                current_term: 0,
                voted_for: 0,
                log: Vec::new(),
                commit_index: 0,
                last_applied_index: 0,
                next_index: Vec::new(),
                match_index: Vec::new(),
            }
        }
    }
    pub fn request_election(mut self) {
        /*
           To begin an election, a follower increments its current
           term and transitions to candidate state. It then votes for
           itself and issues RequestVote RPCs in parallel to each of
           the other servers in the cluster. A candidate continues in
           this state until one of three things happens:
           (a) it wins the election
           (b) another server establishes itself as leaderh
           (c) a period of time goes by with no winner

           Once a candidate wins an election, it becomes leader.
           It then sends heartbeat messages to all of
           the other servers to establish its authority
           and prevent new elections.


        // How is this not exploitable?
        If the leader’s term (included in its RPC) is at least
        as large as the candidate’s current term, then the candidate
        recognizes the leader as legitimate and returns to follower
        state. If the term in the RPC is smaller than the candidate’s
        current term, then the candidate rejects the RPC and continues
        in candidate state
        The third possible outcome is that a candidate neither
        wins nor loses the election:
        if many followers become candidates at the same time, votes could
        be split so that no candidate obtains a majority. When this happens,
        each candidate will time out and start a new election by incrementing
        its term and initiating another round of RequestVote RPCs. However,
        without extra measures split votes could repeat indefinitely
        To prevent split votes in the first place, election timeouts are
        chosen randomly from a fixed interval (e.g., 150–300ms).
        */
        self.state.current_term += 1;
        self.state.role = Role::Candidate;
        // RequestVote
        Server::rpc(
            // self.current_term,
            // self.id,
            // self.last_applied_index,
            // self.last_applied_index,
                   );
    }

    /*
       receiver implementation:
       1. Reply false if term < currentTerm (§5.1)
       2. Reply false if log doesn’t contain an entry at prevLogIndex
       whose term matches prevLogTerm (§5.3)
       3. If an existing entry conflicts with a new one (same index
       but different terms), delete the existing entry and all that
       follow it (§5.3)
       4. Append any new entries not already in the log
       5. If leaderCommit > commitIndex, set commitIndex =
       min(leaderCommit, index of last new entry)
       results:
       term currentTerm, for leader to update itself
       success true if follower contained entry matching
       prevLogIndex and prevLogTerm
       */
    pub fn handle_append_entries(
        self,
        _term: u64,
        _leader: u64,
        _prev_log_index: u64,
        entries: Vec<String>,
        _leader_commit: u64,
        ) -> Result<(), String> {
        // "heartbeat" is append_entries with no log entries
        if entries.len() == 0 {
            return Ok(());
        }
        // self.log.push(..entries);
        Ok(())
    }

    /*
       receiver implementation:
       1. Reply false if term < currentTerm (§5.1)
       2. If votedFor is null or candidateId, and candidate’s log is at
       least as up-to-date as receiver’s log, grant vote (§5.2, §5.4)
       results:
       term currentTerm, for candidate to update itself
       voteGranted true means candidate received vote
       */
    pub fn handle_request_vote(
        mut self,
        term: u64,
        candidate_id: u64,
        _last_log_index: u64,
        _last_log_term: u64,
        ) -> bool {
        if term < self.state.current_term {
            return false;
        }
        if candidate_id != 0 {
            self.state.voted_for = candidate_id;
        }
        // if self.voted_for
        true
    }

    fn rpc() {}
}
