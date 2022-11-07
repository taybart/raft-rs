use capnp::capability::Promise;
use capnp_rpc::{pry, rpc_twoparty_capnp, twoparty, RpcSystem};

use futures::AsyncReadExt;

/*** RAFT ***/
mod raft_capnp {
    include!(concat!(env!("OUT_DIR"), "/proto/raft_capnp.rs"));
}
use crate::raft_capnp::raft;

const ADDRESS: &'static str = "127.0.0.1:9090";

/*
struct ServerImpl;
impl raft::Server for ServerImpl {
    fn request_vote(
        &mut self,
        params: raft::RequestVoteParams,
        mut results: raft::RequestVoteResults,
    ) -> Promise<(), capnp::Error> {
        let request = pry!(pry!(params.get()).get_req());
        println!("{}", request.get_id());
        // results.get();
        Promise::ok(())
    }
    fn append_entries(
        &mut self,
        _params: raft::AppendEntriesParams,
        mut results: raft::AppendEntriesResults,
    ) -> Promise<(), capnp::Error> {
        Promise::ok(())
    }
}
*/

struct RaftImpl;

impl raft::Server for RaftImpl {
    fn say_hello(
        &mut self,
        params: raft::SayHelloParams,
        mut results: raft::SayHelloResults,
    ) -> Promise<(), ::capnp::Error> {
        let request = pry!(pry!(params.get()).get_request());
        let name = pry!(request.get_name());
        let message = format!("Hello, {}!", name);

        results.get().init_reply().set_message(&message);

        Promise::ok(())
    }
}

#[tokio::main]
async fn main() {
    run().await.map_err(|_| "idk what i am doing").unwrap();
    // Ok(())
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tokio::task::LocalSet::new()
        .run_until(async move {
            let listener = tokio::net::TcpListener::bind(ADDRESS).await?;
            let client: raft::Client = capnp_rpc::new_client(RaftImpl);

            loop {
                let (stream, _) = listener.accept().await?;
                stream.set_nodelay(true)?;
                let (reader, writer) =
                    tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
                let network = twoparty::VatNetwork::new(
                    reader,
                    writer,
                    rpc_twoparty_capnp::Side::Server,
                    Default::default(),
                );

                let rpc_system = RpcSystem::new(Box::new(network), Some(client.clone().client));
                tokio::task::spawn_local(rpc_system);
            }
        })
        .await
}
