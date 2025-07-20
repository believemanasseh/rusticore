use rusticore::run_server;
use rusticore::ServerState;

#[test]
fn test_server() {
    let server = run_server(String::from("localhost"), 8080, false, None);
    assert!(server.is_ok(), "Server should start successfully");

    let is_running = server.ok().unwrap().check_state(ServerState::Running).0;
    assert_eq!(is_running, true, "Server should return true on success");
}
