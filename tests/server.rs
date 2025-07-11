use rusticore::run_server;
use rusticore::server::ServerState;

#[test]
fn test_server() {
    let server = run_server(String::from("localhost"), 8080, false, None);
    let is_running = server.check_state(ServerState::Running).0;
    assert_eq!(is_running, true, "Server should return true on success");
}
