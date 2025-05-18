use crate::{
    messages::{Body, Init, InitOk},
    router::Router,
};

pub fn insert_handlers<U>(router: &mut Router<U>) {
    router.on(|init: Init, tx, src, maelstrom, _| {
        maelstrom.node_id = init.node_id;
        let body = Body::InitOk(InitOk {
            in_reply_to: init.msg_id,
        });
        let msg = maelstrom.create_message(src, body);
        tx.send(msg).unwrap();
    });
}

pub fn create_router<U: Default>() -> Router<U> {
    let mut router = Router::default();
    insert_handlers(&mut router);
    router
}

#[cfg(test)]
mod tests {
    use crate::{messages::Body, testing};

    use super::create_router;

    #[test]
    fn should_responde_to_init_with_init_ok() {
        let router = create_router::<()>();
        testing::TestServer::from_router(router)
        .send_str( r#"{"id":0,"src":"c0","dest":"n0","body":{"type":"init","node_id":"n0","node_ids":["n0"],"msg_id":1}}"#)
            .assert_msg_received_default_timeout(|msg| {
                matches!(msg.body, Body::InitOk(_))
            });
    }
}
