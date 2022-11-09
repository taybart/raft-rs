use super::protocol;
use prost::{bytes::Bytes, Message};
use std::io::{Read, Write};
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

pub struct Server {
    pub addr: String,
}

impl Server {
    // pub async fn request_vote(
    //     self,
    //     req: protocol::VoteRequest,
    // ) -> Result<protocol::VoteResponse, String> {
    //     Ok(())
    // }
    pub async fn listen(self) -> Result<(), String> {
        // let addr = format!("{}:{}", host, port);
        let listener =
            std::net::TcpListener::bind(self.addr).map_err(|_| "failed to bind address")?;

        println!("listening...");

        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    println!("Connection accepted");

                    let mut buf = [0; 128];
                    let mut read_bytes = 0;
                    while read_bytes == 0 {
                        read_bytes = s.read(&mut buf).map_err(|_| "failed to read from socket")?;
                        if read_bytes > 0 {
                            println!("received bytes {}", read_bytes);
                        }
                    }

                    let req =
                        protocol::VoteRequest::decode(Bytes::copy_from_slice(&buf[0..read_bytes]))
                            .unwrap();
                    println!("test? {}", req.other_thing);
                }
                Err(e) => {
                    println!("Error while accepting incoming connection - {}", e);
                }
            }
        }

        Ok(())
    }
    // pub async fn listen(self) -> Result<(), String> {
    //     // let addr = format!("{}:{}", host, port);
    //     let listener = tokio::net::TcpListener::bind(self.addr)
    //         .await
    //         .map_err(|_| "failed to bind address")?;

    //     println!("listening...");

    //     let (handle, _) = listener
    //         .accept()
    //         .await
    //         .map_err(|_| "failed to accept incoming connection")?;

    //     let (mut reader, mut writer) = handle.into_split();

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
