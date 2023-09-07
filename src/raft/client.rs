use crate::{
    protocol::{
        AppendRequest, AppendResult, DiscoverResponse, Function, Rpc, Server, VoteRequest,
        VoteResponse,
    },
    raft::FRAME_SIZE,
};
use anyhow::{Context, Result};
use prost::{bytes::Bytes, Message};
use std::time::Duration;

pub async fn add_friend(addr: String, req: Server) -> Result<AppendResult> {
    let b_res = do_rpc(
        addr,
        Rpc {
            call: Function::AddFriend as i32,
            request: req.encode_to_vec(),
        },
    )
    .await?;
    let res = AppendResult::decode(Bytes::copy_from_slice(&b_res))
        .context("failed to decode response")?;
    Ok(res)
}

pub async fn network_discovery(addr: String, req: Server) -> Result<DiscoverResponse> {
    let b_res = do_rpc(
        addr,
        Rpc {
            call: Function::NetworkDiscovery as i32,
            request: req.encode_to_vec(),
        },
    )
    .await?;
    let res = DiscoverResponse::decode(Bytes::copy_from_slice(&b_res))
        .context("failed to decode response")?;
    Ok(res)
}

pub async fn request_vote(addr: String, req: VoteRequest) -> Result<VoteResponse> {
    let b_res = do_rpc(
        addr,
        Rpc {
            call: Function::RequestVote as i32,
            request: req.encode_to_vec(),
        },
    )
    .await?;
    let res = VoteResponse::decode(Bytes::copy_from_slice(&b_res))
        .context("failed to decode response")?;
    Ok(res)
}

pub async fn append_entries(addr: String, req: AppendRequest) -> Result<AppendResult> {
    let b_res = do_rpc(
        addr,
        Rpc {
            call: Function::AppendEntries as i32,
            request: req.encode_to_vec(),
        },
    )
    .await?;
    let res = AppendResult::decode(Bytes::copy_from_slice(&b_res))
        .context("failed to decode response")?;
    Ok(res)
}

async fn do_rpc(addr: String, req: Rpc) -> Result<Vec<u8>> {
    let client = match tokio::time::timeout(
        Duration::from_secs(5),
        tokio::net::TcpStream::connect(addr.clone()),
    )
    .await
    {
        Ok(ok) => ok,
        Err(e) => panic!("client connection timeout {e}"),
    }
    .with_context(|| format!("could not connect to {}", addr.clone()))?;

    client
        .writable()
        .await
        .context("tcp stream not writeable")?;
    client
        .try_write(&req.encode_to_vec())
        .context("failed to write request")?;

    client.readable().await.context("tcp stream not readable")?;
    let mut buf = vec![0; FRAME_SIZE];
    let n = client
        .try_read(&mut buf)
        .context("failed to read response")?;
    buf.truncate(n);
    Ok(buf)
}
