use rusticore::run_server;
use rusticore::ServerState;

#[tokio::test]
async fn test_server() {
    let server = run_server("localhost", 8080, false, None, None).await;
    assert!(server.is_ok(), "Server should start successfully");

    let is_running = server
        .ok()
        .unwrap()
        .check_state(ServerState::Running)
        .await
        .0;
    assert_eq!(is_running, true, "Server should return true on success");
}
