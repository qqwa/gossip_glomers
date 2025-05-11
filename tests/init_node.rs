use common::TestServer;
mod common;

#[test]
fn init_message() {
    TestServer::new()
        .send_str( r#"{"id":0,"src":"c0","dest":"n0","body":{"type":"init","node_id":"n0","node_ids":["n0"],"msg_id":1}}"#)
        .expect_raw_message(r#"{"src":"n0","dest":"c0","body":{"type":"init_ok","in_reply_to":1}}"#);
}
