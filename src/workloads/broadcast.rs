use std::{
    collections::HashMap,
    sync::mpsc::Sender,
    time::{Duration, Instant},
};

use crate::{
    Maelstrom,
    messages::{Body, Broadcast, BroadcastOk, Message, Read, ReadOk, Topology, TopologyOk},
    router::Router,
};

#[derive(Debug, Default)]
pub struct SimpleBroadcast {
    messages: Vec<serde_json::Value>,
    neighbors: Vec<Neighboar>,
    unack_messages: HashMap<u64, (Instant, Message)>,
}

#[derive(Debug, Default, Clone)]
struct Neighboar {
    name: String,
    newest_unack: Option<Instant>,
    newest_ack: Option<Instant>,
}

impl SimpleBroadcast {
    fn store(&mut self, node: &str, msg: Message, msg_id: u64) {
        let timestamp = Instant::now();
        let msg = (timestamp, msg);
        self.unack_messages.insert(msg_id, msg);
        for neighboar in self.neighbors.iter_mut() {
            if neighboar.name == node {
                neighboar.newest_unack = Some(timestamp);
            }
        }
    }

    fn clear(&mut self, node: &str, msg_id: u64) {
        if let Some((timestamp, msg)) = self.unack_messages.remove(&msg_id) {
            for neighboar in self.neighbors.iter_mut() {
                if neighboar.name == node {
                    neighboar.newest_ack = Some(Instant::now());
                }
            }
        }
    }
}

fn tick(tx: &mut Sender<Message>, maelstrom: &mut Maelstrom, data: &mut SimpleBroadcast) {
    for (_, (timestamp, msg)) in data.unack_messages.clone() {
        if Duration::from_millis(200) < timestamp.elapsed() {
            eprintln!("Resending: {:?}", msg);
            tx.send(msg).unwrap();
        }
    }
}

fn broadcast(
    broadcast: Broadcast,
    tx: &mut Sender<Message>,
    src: &str,
    maelstrom: &mut Maelstrom,
    data: &mut SimpleBroadcast,
) {
    // only broadcast message to neighbors if we haven't stored it yet
    if !data.messages.contains(&broadcast.message) {
        let msg_id = maelstrom.generate_id();
        let mut broadcast_neighbors = broadcast.clone();
        broadcast_neighbors.msg_id = msg_id;
        let broadcast_neighbors = Body::Broadcast(broadcast_neighbors);
        data.messages.push(broadcast.message);

        for neighboar in data.neighbors.clone() {
            let msg = maelstrom.create_message(&neighboar.name, broadcast_neighbors.clone());
            data.store(src, msg.clone(), msg_id);
            tx.send(msg).unwrap();
        }
    }

    // always respond with broadcast_ok, so other nodes know we got the message and don't resend
    let body = Body::BroadcastOk(BroadcastOk {
        msg_id: None,
        in_reply_to: broadcast.msg_id,
    });
    let msg = maelstrom.create_message(src, body);
    tx.send(msg).unwrap();
}

fn broadcast_ok(
    broadcast_ok: BroadcastOk,
    tx: &mut Sender<Message>,
    src: &str,
    maelstrom: &mut Maelstrom,
    data: &mut SimpleBroadcast,
) {
    eprintln!("Received from {}: {:?}", src, broadcast_ok);
    data.clear(src, broadcast_ok.in_reply_to);
}

fn read(
    read: Read,
    tx: &mut Sender<Message>,
    src: &str,
    maelstrom: &mut Maelstrom,
    data: &mut SimpleBroadcast,
) {
    let body = Body::ReadOk(ReadOk {
        msg_id: None,
        in_reply_to: read.msg_id,
        messages: data.messages.clone(),
    });
    let msg = maelstrom.create_message(src, body);
    tx.send(msg).unwrap();
}

fn topology(
    topology: Topology,
    tx: &mut Sender<Message>,
    src: &str,
    maelstrom: &mut Maelstrom,
    data: &mut SimpleBroadcast,
) {
    if let Some(neighbors) = topology.topology.get(&maelstrom.node_id) {
        for neighboar in neighbors {
            data.neighbors.push(Neighboar {
                name: neighboar.to_string(),
                newest_unack: None,
                newest_ack: None,
            });
        }

        eprintln!("Topology: {:#?}", data.neighbors);

        let body = Body::TopologyOk(TopologyOk {
            msg_id: None,
            in_reply_to: topology.msg_id,
        });
        let msg = maelstrom.create_message(src, body);
        tx.send(msg).unwrap();
    }
}

pub fn insert_broadcast_simple_handlers(router: &mut Router<SimpleBroadcast>) {
    router.set_tick(tick);

    router.on(broadcast);
    router.on(broadcast_ok);
    router.on(read);
    router.on(topology);
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{messages::Body, testing, workloads::init::create_router};

    use super::{SimpleBroadcast, insert_broadcast_simple_handlers};
    #[test]
    fn should_respond_with_broadcast_ok() {
        let mut router = create_router::<SimpleBroadcast>();
        insert_broadcast_simple_handlers(&mut router);
        testing::TestServer::from_router(router)
            .send_str(
                r#"{"src":"c1","dest":"n1","body":{"type":"broadcast","message":1000,"msg_id":1}}"#,
            )
            .assert_msg_received_default_timeout(|msg| matches!(msg.body, Body::BroadcastOk(_)));
    }

    #[test]
    fn shoud_respond_with_empty_array_on_read_when_no_broadcast_happend_yet() {
        let mut router = create_router::<SimpleBroadcast>();
        insert_broadcast_simple_handlers(&mut router);
        testing::TestServer::from_router(router)
            .send_str(r#"{"src":"c1","dest":"n1","body":{"type":"read","msg_id":1}}"#)
            .assert_msg_received_default_timeout(|msg| {
                if let Body::ReadOk(read_ok) = &msg.body {
                    read_ok.messages.is_empty()
                } else {
                    false
                }
            });
    }

    #[test]
    fn shoud_respond_with_value_on_read_when_broadcast_happend() {
        let mut router = create_router::<SimpleBroadcast>();
        insert_broadcast_simple_handlers(&mut router);
        testing::TestServer::from_router(router)
            .send_str(
                r#"{"src":"c1","dest":"n1","body":{"type":"broadcast","message":1000,"msg_id":1}}"#,
            )
            .send_str(r#"{"src":"c1","dest":"n1","body":{"type":"read","msg_id":1}}"#)
            .assert_msg_received_default_timeout(|msg| {
                if let Body::ReadOk(read_ok) = &msg.body {
                    read_ok.messages.len() == 1 && read_ok.messages[0] == json!(1000)
                } else {
                    false
                }
            });
    }

    #[test]
    fn shoud_respond_only_return_unique_messages_on_read_when_broadcast_happend() {
        let mut router = create_router::<SimpleBroadcast>();
        insert_broadcast_simple_handlers(&mut router);
        testing::TestServer::from_router(router)
            .send_str(
                r#"{"src":"c1","dest":"n1","body":{"type":"broadcast","message":1000,"msg_id":1}}"#,
            )
            .send_str(
                r#"{"src":"c1","dest":"n1","body":{"type":"broadcast","message":1000,"msg_id":2}}"#,
            )
            .send_str(r#"{"src":"c1","dest":"n1","body":{"type":"read","msg_id":3}}"#)
            .assert_msg_received_default_timeout(|msg| {
                if let Body::ReadOk(read_ok) = &msg.body {
                    read_ok.messages.len() == 1 && read_ok.messages[0] == json!(1000)
                } else {
                    false
                }
            });
    }

    #[test]
    fn topology_message() {
        let mut router = create_router::<SimpleBroadcast>();
        insert_broadcast_simple_handlers(&mut router);
        testing::TestServer::from_router(router)
        .send_str( r#"{"id":0,"src":"c0","dest":"n0","body":{"type":"init","node_id":"n0","node_ids":["n0"],"msg_id":1}}"#)
            .send_str(r#"{"src":"c1","dest":"n0","body":{"type":"topology","topology":{"n0":[]},"msg_id":1}}"#)
            .assert_msg_received_default_timeout(|msg| matches!(msg.body, Body::TopologyOk(_)));
    }

    #[test]
    fn forward_broadcast_messages() {
        let mut router = create_router::<SimpleBroadcast>();
        insert_broadcast_simple_handlers(&mut router);
        testing::TestServer::from_router(router)
            .send_str( r#"{"id":0,"src":"c0","dest":"n0","body":{"type":"init","node_id":"n0","node_ids":["n0"],"msg_id":1}}"#)
            .send_str(r#"{"src":"c1","dest":"n0","body":{"type":"topology","topology":{"n0":["n1"]},"msg_id":1}}"#)
            .send_str(
            r#"{"src":"c1","dest":"n1","body":{"type":"broadcast","message":"text","msg_id":1}}"#,
        )
            .assert_msg_received_default_timeout(|msg| {
                if let Body::Broadcast(broadcast) = &msg.body {
                    msg.dest == "n1" && broadcast.message == json!("text")
                } else {
                    false
                }
            });
    }
}
