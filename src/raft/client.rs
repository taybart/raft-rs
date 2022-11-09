use super::protocol;
use prost::Message;
use std::io::Write;
use std::net::TcpStream;

pub struct Client {
    pub addr: String,
}

impl Client {
    pub async fn request_vote(
        self,
        req: protocol::VoteRequest,
    ) -> Result<protocol::VoteResponse, String> {
        let _b_res = self.call(req.encode_to_vec()).unwrap();
        Ok(protocol::VoteResponse::default())
    }

    fn call(self, req: Vec<u8>) -> Result<Vec<u8>, String> {
        let mut client = TcpStream::connect(self.addr).map_err(|_| "failed to connect")?;

        println!("connected...");

        client
            .write_all(&req)
            .map_err(|_| "failed to write request")?;

        println!("wrote {} bytes", req.len());

        let buf = Vec::new();

        // client
        //     .read_to_end(&mut buf)
        //     .map_err(|_| "failed to write request")?;

        Ok(buf)
    }

    // async fn call(self) -> Result<Vec<u8>, String> {
    //     // let addr = format!("{}:{}", host, port);
    //     let client = tokio::net::TcpStream::connect(self.addr)
    //         .await
    //         .map_err(|_| "failed to connect")?;

    //     let (mut reader, mut writer) = client.into_split();

    //     let client_read =
    //         tokio::spawn(
    //             async move { tokio::io::copy(&mut reader, &mut tokio::io::stdout()).await },
    //         );

    //     let client_write =
    //         tokio::spawn(
    //             async move { tokio::io::copy(&mut tokio::io::stdin(), &mut writer).await },
    //         );

    //     tokio::select! {
    //         _ = client_read => {}
    //         _ = client_write => {}
    //     }

    //     Ok(())
    // }
}
