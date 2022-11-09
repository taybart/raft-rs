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
            c.request_vote(protocol::VoteRequest {
                other_thing: "woot".to_string(),
                term: 1,
                candidate_id: 0,
                last_log_index: 0,
                last_log_term: 0,
            })
            .await
            .unwrap();
        }
        cli::Commands::Serve { host, port } => {
            let addr = format!("{}:{}", host, port);
            let s = server::Server { addr };
            s.listen().await;
        }
    }
}
