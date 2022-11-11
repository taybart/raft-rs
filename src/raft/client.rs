use super::protocol::{AppendRequest, AppendResult, Function, Rpc, VoteRequest, VoteResponse};
use prost::{bytes::Bytes, Message};
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct Client {
    pub addr: String,
}

impl Client {
    pub async fn request_vote(&self, req: VoteRequest) -> Result<VoteResponse, String> {
        let mut rpc = Rpc::default();
        rpc.set_call(Function::RequestVote);
        rpc.request = req.encode_to_vec();
        let b_res = self.call(rpc)?;
        let res = VoteResponse::decode(Bytes::copy_from_slice(&b_res))
            .map_err(|_| "failed to decode response")?;
        Ok(res)
    }
    pub async fn append_entries(&self, req: AppendRequest) -> Result<AppendResult, String> {
        let mut rpc = Rpc::default();
        rpc.set_call(Function::AppendEntries);
        rpc.request = req.encode_to_vec();
        let b_res = self.call(rpc)?;
        let res = AppendResult::decode(Bytes::copy_from_slice(&b_res))
            .map_err(|_| "failed to decode response")?;
        Ok(res)
    }

    fn call(&self, req: Rpc) -> Result<Vec<u8>, String> {
        let mut client = TcpStream::connect(self.addr.clone()).map_err(|_| "failed to connect")?;

        client
            .write_all(&req.encode_to_vec())
            .map_err(|_| "failed to write request")?;

        let mut buf = vec![0; 4096];
        let n = client
            .read(&mut buf)
            .map_err(|_| "failed to read response")?;
        buf.truncate(n);
        Ok(buf)
    }
}
