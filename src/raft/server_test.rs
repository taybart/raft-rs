#[cfg(test)]
mod tests {
    use crate::server::Server;

    #[tokio::test]
    async fn lonely_server() {
        Server {
            addr: "127.0.0.1:9090".to_string(),
            ..Default::default()
        }
        .listen(
            vec!["http://127.0.0.1:9091".to_string()],
            crate::raft::roles::Role::Follower,
        )
        .await
        .unwrap();
        // sleep(Duration::from_millis(1000)).await;
    }
}
