use super::protocol;
use prost::Message;

pub async fn request_vote(req: protocol::VoteRequest) -> Result<protocol::VoteResponse, String> {
    let mut res = protocol::VoteResponse::default();
    res.term = req.term;
    let mut buf = Vec::new();
    res.encode(&mut buf).map_err(|_| "issue")?;
    println!("test? {}", res.term);
    Ok(res)
}

// pub async fn request_vote(req: protocol::VoteRequest) -> Result<protocol::VoteResponse, String> {
//     let mut res = protocol::VoteResponse::default();
//     res.term = req.term;
//     let mut buf = Vec::new();
//     res.encode(&mut buf).map_err(|_| "issue")?;
//     // Ok(buf)
//     // let res = raft::VoteResponse::decode(req.as_slice()).unwrap();

//     println!("test? {}", res.term);
//     Ok(res)
// }

// pub async fn connect(host: &String, port: &u16) -> Result<(), String> {
//     let addr = format!("{}:{}", host, port);
//     let client = tokio::net::TcpStream::connect(addr)
//         .await
//         .map_err(|_| "failed to connect")?;

//     let (mut reader, mut writer) = client.into_split();

//     let client_read =
//         tokio::spawn(async move { tokio::io::copy(&mut reader, &mut tokio::io::stdout()).await });

//     let client_write =
//         tokio::spawn(async move { tokio::io::copy(&mut tokio::io::stdin(), &mut writer).await });

//     tokio::select! {
//         _ = client_read => {}
//         _ = client_write => {}
//     }

//     Ok(())
// }

// pub async fn listen(host: &String, port: &u16) -> Result<(), String> {
//     let addr = format!("{}:{}", host, port);
//     let listener = tokio::net::TcpListener::bind(addr)
//         .await
//         .map_err(|_| "failed to bind address")?;

//     let (handle, _) = listener
//         .accept()
//         .await
//         .map_err(|_| "failed to accept incoming connection")?;

//     let (mut reader, mut writer) = handle.into_split();

//     let client_read =
//         tokio::spawn(async move { tokio::io::copy(&mut reader, &mut tokio::io::stdout()).await });

//     let client_write =
//         tokio::spawn(async move { tokio::io::copy(&mut tokio::io::stdin(), &mut writer).await });

//     tokio::select! {
//         _ = client_read => {}
//         _ = client_write => {}
//     }

//     Ok(())
// }
