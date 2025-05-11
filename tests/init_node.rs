use std::io::BufRead;

use common::TestServer;
use gossip_glomers::Server;
mod common;

#[test]
fn init() {
    let msg = r#"{"id":0,"src":"c0","dest":"n0","body":{"type":"init","node_id":"n0","node_ids":["n0"],"msg_id":1}}"#;
    let msg = msg.to_string() + "\n";
    let mut server = Server::new();
    let mut output = Vec::new();
    server.serve(&mut msg.as_bytes(), &mut output, &mut Vec::new());
    let got = output.lines().map(|l| l.unwrap()).collect::<Vec<String>>();
    let want = vec![r#"{"src":"n0","dest":"c0","body":{"type":"init_ok","in_reply_to":1}}"#];
    assert_eq!(got, want);
}

#[test]
fn init_message() {
    TestServer::new()
        .send_str( r#"{"id":0,"src":"c0","dest":"n0","body":{"type":"init","node_id":"n0","node_ids":["n0"],"msg_id":1}}"#)
        .expect_raw_message(r#"{"src":"n0","dest":"c0","body":{"type":"init_ok","in_reply_to":1}}"#);
}
