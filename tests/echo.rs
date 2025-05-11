use common::TestServer;
mod common;

#[test]
fn echo_message() {
    TestServer::new()
        .send_str( r#"{"src":"c1","dest":"n1","body":{"type":"echo","msg_id":1,"echo":"Please echo 35"}}"#)
        .close() // prevents test to get stuck
        .expect_raw_message(r#"{"src":"n1","dest":"c1","body":{"type":"echo_ok","msg_id":1,"in_reply_to":1,"echo":"Please echo 35"}}"#);
}
