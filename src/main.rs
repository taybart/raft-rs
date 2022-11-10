use clap::Parser;
use raft::{client, protocol, server};

pub mod cli;
mod raft;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();

    match &args.command {
        cli::Commands::Connect { host, port } => {
            let addr = format!("{}:{}", host, port);
            let c = client::Client { addr };
            let rv_res = c
                .request_vote(protocol::VoteRequest {
                    term: 69,
                    candidate_id: 0,
                    last_log_index: 0,
                    last_log_term: 0,
                })
                .await
                .unwrap();
            println!(
                "vote granted: {} in term {}",
                rv_res.vote_grated, rv_res.term
            );
            let ae_res = c
                .append_entries(protocol::AppendRequest {
                    id: 0,
                    term: 69,
                    leader_id: 0,
                    entries: vec![],
                    prev_log_term: 0,
                    prev_log_index: 0,
                })
                .await
                .unwrap();
            println!(
                "heartbeat {}successful in term {}",
                if !ae_res.success { "un" } else { "" },
                ae_res.term
            )
        }
        cli::Commands::Serve { host, port } => {
            let addr = format!("{}:{}", host, port);
            let s = server::Server { addr };
            match s.listen().await {
                Ok(()) => {}
                Err(e) => {
                    println!("server error: {}", e)
                }
            }
        }
    }
}
