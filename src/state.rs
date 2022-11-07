// Servers can be in 1 of 3 states: leader, follower, or candidate
//
//              <- discovers new leader or term
//                ┌───────────┐
// starts -> follower -> candidate -> leader

// If a follower receives no communication over a period of time called the election timeout,
// then it assumes there is no viable leader and begins an election to choose a new leader
pub const ELECTION_TIMEOUT: u32 = 10_000;

pub struct State {
    // Persistent (updated on stable storage before responding to RPCs)
    current_term: u64, // monotonic
    voted_for: String,
    log: Vec<String>,

    // Voliatile (all servers)
    commit_index: u64,
    last_applied: u64,

    // Voliatile (leaders) (reinitialized after election)
    next_index: Vec<u64>,
    match_index: Vec<u64>,
}
