use super::protocol::{
    AppendRequest, AppendResult, DiscoverResponse, Function, Rpc, Server, VoteRequest, VoteResponse,
};
use super::FRAME_SIZE;
use prost::{bytes::Bytes, Message};
use std::time::Duration;

pub async fn add_friend(addr: String, req: Server) -> Result<AppendResult, String> {
    let b_res = do_rpc(
        addr,
        Rpc {
            call: Function::AddFriend as i32,
            request: req.encode_to_vec(),
        },
    )
    .await?;
    let res = AppendResult::decode(Bytes::copy_from_slice(&b_res))
        .map_err(|_| "failed to decode response")?;
    Ok(res)
}
pub async fn network_discovery(addr: String, req: Server) -> Result<DiscoverResponse, String> {
    let b_res = do_rpc(
        addr,
        Rpc {
            call: Function::NetworkDiscovery as i32,
            request: req.encode_to_vec(),
        },
    )
    .await?;
    let res = DiscoverResponse::decode(Bytes::copy_from_slice(&b_res))
        .map_err(|_| "failed to decode response")?;
    Ok(res)
}
pub async fn request_vote(addr: String, req: VoteRequest) -> Result<VoteResponse, String> {
    let b_res = do_rpc(
        addr,
        Rpc {
            call: Function::RequestVote as i32,
            request: req.encode_to_vec(),
        },
    )
    .await?;
    let res = VoteResponse::decode(Bytes::copy_from_slice(&b_res))
        .map_err(|_| "failed to decode response")?;
    Ok(res)
}

pub async fn append_entries(addr: String, req: AppendRequest) -> Result<AppendResult, String> {
    let b_res = do_rpc(
        addr,
        Rpc {
            call: Function::AppendEntries as i32,
            request: req.encode_to_vec(),
        },
    )
    .await?;
    let res = AppendResult::decode(Bytes::copy_from_slice(&b_res))
        .map_err(|_| "failed to decode response")?;
    Ok(res)
}

async fn do_rpc(addr: String, req: Rpc) -> Result<Vec<u8>, String> {
    let client = match tokio::time::timeout(
        Duration::from_secs(5),
        tokio::net::TcpStream::connect(addr.clone()),
    )
    .await
    {
        Ok(ok) => ok,
        Err(e) => panic!("client connection timeout {}", e),
    }
    .map_err(|e| format!("could not connect to {} {}", addr.clone(), e))?;

    client
        .writable()
        .await
        .map_err(|e| format!("tcp stream not writeable {}", e))?;
    client
        .try_write(&req.encode_to_vec())
        .map_err(|_| "failed to write request")?;

    client
        .readable()
        .await
        .map_err(|e| format!("tcp stream not readable {}", e))?;
    let mut buf = vec![0; FRAME_SIZE];
    let n = client
        .try_read(&mut buf)
        .map_err(|_| "failed to read response")?;
    buf.truncate(n);
    Ok(buf)
}
