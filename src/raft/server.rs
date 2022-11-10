use super::protocol;
use prost::{bytes::Bytes, Message};
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    pub addr: String,
}

impl Server {
    pub fn handle_append_entries(req: protocol::AppendRequest) {
        println!("Vote request in term {}!", req.term);
    }
    pub fn handle_vote_request(req: protocol::VoteRequest) {
        println!("Vote request in term {}!", req.term);
    }

    async fn process(stream: TcpStream) -> Result<(), String> {
        println!("Connection accepted");

        let mut buf = vec![0; 1024];
        let _n = match stream.try_read(&mut buf) {
            Ok(n) => buf.truncate(n),
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e)
            }
        };
        let rpc_req = protocol::Rpc::decode(Bytes::copy_from_slice(&buf)).unwrap();
        match protocol::rpc::Func::from_i32(rpc_req.call) {
            Some(protocol::rpc::Func::RequestVote) => {
                let req = protocol::VoteRequest::decode(Bytes::copy_from_slice(&rpc_req.request))
                    .unwrap();
                Self::handle_vote_request(req);
            }
            Some(protocol::rpc::Func::AppendEntries) => {
                let req = protocol::AppendRequest::decode(Bytes::copy_from_slice(&rpc_req.request))
                    .unwrap();
                Self::handle_append_entries(req);
            }
            None => {
                eprintln!("unknown rpc")
            }
        }
        Ok(())
    }
    pub async fn listen(self) -> Result<(), String> {
        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(|_| "failed to bind address")?;

        println!("listening...");
        loop {
            let (socket, _) = listener
                .accept()
                .await
                .map_err(|_| "failed to accept incoming connection")?;
            tokio::spawn(async move {
                let _ = Self::process(socket).await;
            });
        }
    }
}
