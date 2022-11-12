/*
 * should only need to respond when a server starts
 * leader should exist on all states unless weak start
 */
use super::network::FRAME_SIZE;
use super::protocol::{DiscoverRequest, DiscoverResponse};
use super::protocol::{Function, Rpc};

use prost::{bytes::Bytes, Message};
use tokio::{
    net::{TcpListener, TcpStream},
    time::{sleep, Duration},
};

/*
 * process: take discoveries rpc and translate/run it
 */
async fn process(stream: TcpStream) -> Result<(), String> {
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
    let req = Rpc::decode(Bytes::copy_from_slice(&buf)).unwrap();
    match Function::from_i32(req.call) {
        Some(Function::NetworkDiscovery) => {}
        // TODO: serve should not panic
        None => {
            panic!("unknown rpc")
        }
    };

    let res = DiscoverResponse {
        ..Default::default()
    }
    .encode_to_vec();

    stream.try_write(&res).expect("writing response failed");
    Ok(())
}

pub async fn listen(addr: String, id: u64) -> Result<(), String> {
    let listener = TcpListener::bind(addr.clone())
        .await
        .expect("failed to bind address");

    println!("disco: listening on {}...", addr);
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
