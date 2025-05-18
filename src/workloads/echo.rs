use crate::{
    messages::{Body, Echo, EchoOk},
    router::Router,
};

pub fn insert_echo_handlers<U>(router: &mut Router<U>) {
    router.on(|echo: Echo, tx, src, maelstrom, _| {
        let body = Body::EchoOk(EchoOk {
            msg_id: None,
            in_reply_to: echo.msg_id,
            echo: echo.echo,
        });
        let msg = maelstrom.create_message(src, body);
        tx.send(msg).unwrap();
    });
}

#[cfg(test)]
mod tests {
    use crate::{messages::Body, testing, workloads::init::create_router};

    use super::insert_echo_handlers;

    #[test]
    fn should_echo_msg_back_with_echo_ok() {
        let mut router = create_router::<()>();
        insert_echo_handlers(&mut router);
        testing::TestServer::from_router(router)
        .send_str( r#"{"src":"c1","dest":"n1","body":{"type":"echo","msg_id":1,"echo":"Please echo 35"}}"#)
            .assert_msg_received_default_timeout(|msg| {
                if let Body::EchoOk(echo_ok) = &msg.body {
                    return echo_ok.echo == "Please echo 35";
                }
                false
            });
    }
}
