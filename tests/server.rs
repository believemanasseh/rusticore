use rusticore::run_server;

#[test]
fn test_server() {
    let result = run_server(String::from("localhost"), 8080, false, None);
    assert_eq!(result, true, "Server should return true on success");
}
