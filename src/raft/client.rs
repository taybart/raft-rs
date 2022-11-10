use super::protocol;
use prost::{bytes::Bytes, Message};
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct Client {
    pub addr: String,
}

impl Client {
    pub async fn request_vote(
        &self,
        req: protocol::VoteRequest,
    ) -> Result<protocol::VoteResponse, String> {
        let mut rpc = protocol::Rpc::default();
        rpc.set_call(protocol::rpc::Func::RequestVote);
        rpc.request = req.encode_to_vec();
        let b_res = self.call(rpc).unwrap();
        let res = protocol::VoteResponse::decode(Bytes::copy_from_slice(&b_res))
            .map_err(|_| "failed to decode response")?;
        Ok(res)
    }
    pub async fn append_entries(
        &self,
        req: protocol::AppendRequest,
    ) -> Result<protocol::AppendResult, String> {
        let mut rpc = protocol::Rpc::default();
        rpc.set_call(protocol::rpc::Func::AppendEntries);
        rpc.request = req.encode_to_vec();
        let b_res = self.call(rpc).unwrap();
        let res = protocol::AppendResult::decode(Bytes::copy_from_slice(&b_res))
            .map_err(|_| "failed to decode response")?;
        Ok(res)
    }

    fn call(&self, req: protocol::Rpc) -> Result<Vec<u8>, String> {
        let mut client = TcpStream::connect(self.addr.clone()).map_err(|_| "failed to connect")?;

        println!("connected...");

        client
            .write_all(&req.encode_to_vec())
            .map_err(|_| "failed to write request")?;

        let mut buf = vec![0; 1024];
        let n = client
            .read(&mut buf)
            .map_err(|_| "failed to read response")?;
        buf.truncate(n);
        Ok(buf)
    }
}
