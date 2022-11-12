use clap::Parser;
use raft::{client, protocol, roles, server};

pub mod cli;
mod raft;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();

    match &args.command {
        cli::Commands::Connect { host, port, rpc } => {
            let addr = format!("{}:{}", host, port);
            // encapulate better
            match rpc.as_str() {
                "request_vote" => {
                    let rv_res = client::request_vote(
                        addr,
                        protocol::VoteRequest {
                            term: 69,
                            candidate_id: 0,
                            last_log_index: 0,
                            last_log_term: 0,
                        },
                    )
                    .await
                    .unwrap();
                    println!(
                        "vote granted: {} in term {}",
                        rv_res.vote_granted, rv_res.term
                    );
                }
                "ping" => {
                    let ae_res_1 = client::append_entries(
                        addr,
                        protocol::AppendRequest {
                            id: 0,
                            term: 69,
                            leader_id: 0,
                            entries: vec![],
                            prev_log_term: 0,
                            prev_log_index: 0,
                        },
                    )
                    .await
                    .unwrap();
                    println!(
                        "heartbeat {}successful in term {}",
                        if !ae_res_1.success { "un" } else { "" },
                        ae_res_1.term
                    );
                }
                "append_entries" => {
                    let mut handles = Vec::new();
                    for i in 0..100 {
                        let addr = addr.clone();
                        handles.push(tokio::spawn(async move {
                            let res = client::append_entries(
                                addr,
                                protocol::AppendRequest {
                                    id: i,
                                    term: 0,
                                    leader_id: 0,
                                    entries: vec![protocol::LogEntry {
                                        data: "test".to_string(),
                                    }],
                                    prev_log_term: 0,
                                    prev_log_index: 0,
                                },
                            )
                            .await
                            .expect("issue appending entries");
                            if !res.success {
                                eprintln!("appending log entries unsuccessful")
                            }
                        }));
                    }
                    for handle in handles {
                        handle.await.expect("panic in task");
                    }
                }
                _ => {
                    eprintln!("unknown rpc must be one of [request_vote, ping, append_entries]")
                }
            }
        }
        cli::Commands::Serve { host, port, role } => {
            let addr = format!("{}:{}", host, port);

            let role = roles::Role::from_string(role.clone()).unwrap();

            // this should be generated also,
            // discovery should be built into the protocol
            let friends = match *port {
                9090 => [
                    "127.0.0.1:9091",
                    "127.0.0.1:9092",
                    "127.0.0.1:9093",
                    "127.0.0.1:9094",
                ],
                9091 => [
                    "127.0.0.1:9090",
                    "127.0.0.1:9092",
                    "127.0.0.1:9093",
                    "127.0.0.1:9094",
                ],
                9092 => [
                    "127.0.0.1:9090",
                    "127.0.0.1:9091",
                    "127.0.0.1:9093",
                    "127.0.0.1:9094",
                ],
                9093 => [
                    "127.0.0.1:9090",
                    "127.0.0.1:9091",
                    "127.0.0.1:9092",
                    "127.0.0.1:9094",
                ],
                9094 => [
                    "127.0.0.1:9090",
                    "127.0.0.1:9091",
                    "127.0.0.1:9092",
                    "127.0.0.1:9093",
                ],
                _ => panic!("unknown server id"),
            };
            tokio::select! {
                res = server::listen(
                    addr,
                    *port as u64,
                    friends.to_vec().clone(),
                    role,
                    ) => {
                    if let Err(e) = res {
                        println!("listen failed: {}", e.to_string());
                    }
                }
                _ = tokio::signal::ctrl_c() => {}
            }
        }
    }
}
