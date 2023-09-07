/*
 * should only need to respond when a server starts
 * leader should exist on all states unless weak start
 */
use crate::{
    protocol::{DiscoverResponse, Function, RaftErr, Rpc, Server},
    raft,
    raft::FRAME_SIZE,
};

use anyhow::{Context, Result};
use prost::{bytes::Bytes, Message};
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

type SState = Arc<Mutex<State>>;
#[derive(Default, Clone)]
pub struct State {
    in_network: Vec<Server>,
}

/*
 * handle_discovery: should take one of two things(
 * 1. no nodes on the network
 * 2. return entire network (new node should announce)
 * )
 */
pub async fn handle_discovery(state: SState, req: Server) -> Vec<u8> {
    let mut ret = DiscoverResponse {
        ..Default::default()
    };
    let mut in_network = false;
    let mut friends = Vec::new();
    // grab state snapshot
    if let Ok(mut s) = state.lock() {
        // check if this is a new node
        if !s.in_network.is_empty() {
            for server in s.clone().in_network {
                if server.id == req.id {
                    in_network = true;
                    break;
                }
            }
        }
        // add new node to state
        if !in_network {
            println!("new friend! {}", req.id);
            // snapshot network
            friends = s.in_network.clone();
            // update
            s.in_network.push(Server {
                id: req.id,
                addr: req.addr.clone(),
            });
        }
        ret.friends = s.in_network.clone();
    }

    // if this is a new node broadcast to all current nodes
    if !in_network {
        for friend in friends {
            raft::client::add_friend(
                friend.addr,
                Server {
                    id: req.id,
                    addr: req.addr.clone(),
                },
            )
            .await
            .unwrap();
        }
    }
    ret.encode_to_vec()
}
/*
 * process: take discoveries rpc and translate/run it
 * TODO: return network error
 */
async fn process(state: SState, stream: TcpStream) -> Result<()> {
    stream.readable().await.context("stream not readable")?;

    let mut buf = vec![0; FRAME_SIZE];
    match stream.try_read(&mut buf) {
        Ok(n) => buf.truncate(n),
        Err(e) => {
            // TODO: send real error out
            eprintln!("failed to read from socket: {e}")
        }
    };
    let req = Rpc::decode(Bytes::copy_from_slice(&buf)).unwrap();
    match Function::from_i32(req.call) {
        Some(Function::NetworkDiscovery) => {
            match Server::decode(Bytes::copy_from_slice(&req.request)) {
                Ok(req) => {
                    let res = handle_discovery(state, req).await;

                    stream.try_write(&res).expect("writing response failed");
                }
                Err(e) => {
                    stream
                        .try_write(
                            &RaftErr {
                                msg: format!("wtf dumb dumb {e}"),
                            }
                            .encode_to_vec(),
                        )
                        .expect("writing response failed");
                }
            };
        }
        None => {
            stream
                .try_write(
                    &RaftErr {
                        msg: "wtf kind of rpc is that?".to_string(),
                    }
                    .encode_to_vec(),
                )
                .expect("writing response failed");
        }
        _ => {}
    };

    Ok(())
}

pub async fn listen(addr: String) -> Result<()> {
    let listener = TcpListener::bind(addr.clone())
        .await
        .expect("failed to bind address");

    // init state, election timeout is randomized to help prevent stalemates
    let state = Arc::new(Mutex::new(State {
        ..Default::default()
    }));

    println!("discovery: listening on {addr}...");
    loop {
        let (socket, _) = listener
            .accept()
            .await
            .context("failed to accept incoming connection")?;

        let state = state.clone();
        tokio::spawn(async move {
            process(state, socket).await.unwrap();
        });
    }
}
