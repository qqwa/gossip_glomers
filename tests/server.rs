use std::collections::HashSet;

use common::TestServer;
use gossip_glomers::messages::Body;
mod common;

#[test]
fn init_message() {
    TestServer::new()
        .send_str( r#"{"id":0,"src":"c0","dest":"n0","body":{"type":"init","node_id":"n0","node_ids":["n0"],"msg_id":1}}"#)
        .expect_raw_message(r#"{"src":"n0","dest":"c0","body":{"type":"init_ok","in_reply_to":1}}"#);
}

#[test]
fn echo_message() {
    TestServer::new()
        .send_str( r#"{"src":"c1","dest":"n1","body":{"type":"echo","msg_id":1,"echo":"Please echo 35"}}"#)
        .expect_raw_message(r#"{"src":"n1","dest":"c1","body":{"type":"echo_ok","in_reply_to":1,"echo":"Please echo 35"}}"#);
}

#[test]
fn generate_message() {
    let mut server = TestServer::new()
        .send_str(r#"{"src":"c1","dest":"n1","body":{"type":"generate","msg_id":1}}"#)
        .send_str(r#"{"src":"c1","dest":"n1","body":{"type":"generate","msg_id":1}}"#)
        .send_str(r#"{"src":"c1","dest":"n1","body":{"type":"generate","msg_id":1}}"#)
        .close();

    let ids: Vec<_> = server
        .get_parsed_messages()
        .into_iter()
        .filter_map(|msg| {
            if let Body::GenerateOk { id, .. } = msg.body {
                Some(id)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(ids.len(), 3);
    assert_eq!(ids.iter().collect::<HashSet<_>>().len(), ids.len());
}

#[test]
fn broadcast_message() {
    TestServer::new()
        .send_str(
            r#"{"src":"c1","dest":"n1","body":{"type":"broadcast","message":1000,"msg_id":1}}"#,
        )
        .expect_raw_message(
            r#"{"src":"n1","dest":"c1","body":{"type":"broadcast_ok","in_reply_to":1}}"#,
        );
}

#[test]
fn read_message_empty() {
    TestServer::new()
        .send_str(r#"{"src":"c1","dest":"n1","body":{"type":"read","msg_id":1}}"#)
        .expect_raw_message(
            r#"{"src":"n1","dest":"c1","body":{"type":"read_ok","in_reply_to":1,"messages":[]}}"#,
        );
}
