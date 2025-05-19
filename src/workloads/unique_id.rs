use uuid::Uuid;

use crate::{
    messages::{Body, Generate, GenerateOk},
    router::Router,
};

pub fn insert_unique_id_handlers<U>(router: &mut Router<U>) {
    router.on(|generate: Generate, tx, src, maelstrom, _| {
        let body = Body::GenerateOk(GenerateOk {
            msg_id: None,
            in_reply_to: generate.msg_id,
            id: Uuid::new_v4().to_string(),
        });
        let msg = maelstrom.create_message(src, body);
        tx.send(msg).unwrap();
    });
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{
        messages::{Body, GenerateOk},
        testing,
        workloads::init::create_router,
    };

    use super::insert_unique_id_handlers;

    #[test]
    fn should_receive_different_ids_from_generate() {
        let mut router = create_router::<()>();
        insert_unique_id_handlers(&mut router);
        let server = testing::TestServer::from_router(router)
            .send_str(r#"{"src":"c1","dest":"n1","body":{"type":"generate","msg_id":1}}"#)
            .send_str(r#"{"src":"c1","dest":"n1","body":{"type":"generate","msg_id":1}}"#)
            .send_str(r#"{"src":"c1","dest":"n1","body":{"type":"generate","msg_id":1}}"#)
            .wait_for_messages();

        let ids: Vec<_> = server
            .get_messages()
            .into_iter()
            .filter_map(|msg| {
                if let Body::GenerateOk(GenerateOk { id, .. }) = &msg.body {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(ids.len(), 3);
        assert_eq!(ids.iter().collect::<HashSet<_>>().len(), ids.len());
    }
}
