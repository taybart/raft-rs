use clap::Parser;
use raft::{client, protocol, server};

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
        cli::Commands::Serve { host, port } => {
            let addr = format!("{}:{}", host, port);

            tokio::select! {
                res = server::listen(
                    addr,
                    *port as u64, // "id"
                    // friends.to_vec().clone(),
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
