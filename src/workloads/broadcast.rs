use crate::{
    messages::{Body, Broadcast, BroadcastOk, Read, ReadOk, Topology, TopologyOk},
    router::Router,
};

#[derive(Debug, Default)]
pub struct SimpleBroadcast {
    messages: Vec<serde_json::Value>,
    neighbors: Vec<String>,
}

pub fn insert_broadcast_simple_handlers(router: &mut Router<SimpleBroadcast>) {
    router.on(|broadcast: Broadcast, tx, src, maelstrom, data| {
        if !data.messages.contains(&broadcast.message) {
            let broadcast_neighbors = Body::Broadcast(broadcast.clone());
            data.messages.push(broadcast.message);

            for neighboar in &data.neighbors {
                tx.send(maelstrom.create_message(&neighboar, broadcast_neighbors.clone()))
                    .unwrap();
            }
        }
        let body = Body::BroadcastOk(BroadcastOk {
            msg_id: None,
            in_reply_to: broadcast.msg_id,
        });
        let msg = maelstrom.create_message(src, body);
        tx.send(msg).unwrap();
    });
    router.on(|read: Read, tx, src, maelstrom, data| {
        let body = Body::ReadOk(ReadOk {
            msg_id: None,
            in_reply_to: read.msg_id,
            messages: data.messages.clone(),
        });
        let msg = maelstrom.create_message(src, body);
        tx.send(msg).unwrap();
    });
    router.on(|topology: Topology, tx, src, maelstrom, data| {
        if let Some(neighbors) = topology.topology.get(&maelstrom.node_id) {
            data.neighbors = neighbors.to_vec();

            let body = Body::TopologyOk(TopologyOk {
                msg_id: None,
                in_reply_to: topology.msg_id,
            });
            let msg = maelstrom.create_message(src, body);
            tx.send(msg).unwrap();
        }
    });
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
