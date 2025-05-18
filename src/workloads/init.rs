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
