use std::io::{BufRead, BufReader, Read, Write};

use messages::Message;

mod messages;
pub struct Server {}

impl Server {
    pub fn new() -> Server {
        Server {}
    }

    pub fn serve(
        &mut self,
        input: &mut dyn Read,
        mut output: &mut dyn Write,
        mut _log: &mut dyn Write,
    ) {
        let input = BufReader::new(input);
        for line in input.lines() {
            let Ok(line) = line else {
                continue;
            };
            let Ok(message) = serde_json::from_str::<Message>(&line) else {
                continue;
            };
            // writeln!(&mut log, "{}", line).unwrap();
            match message.body {
                messages::Body::Init {
                    msg_id,
                    node_id: _,
                    node_ids: _,
                } => {
                    let reply = message.create_response(messages::Body::InitOk {
                        in_reply_to: msg_id,
                    });
                    serde_json::to_writer(&mut output, &reply).unwrap();
                    writeln!(&mut output).unwrap();
                    // output.flush().unwrap();
                }
                messages::Body::InitOk { in_reply_to: _ } => todo!(),
                messages::Body::Echo { msg_id, ref echo } => {
                    let reply = message.create_response(messages::Body::EchoOk {
                        msg_id: None,
                        in_reply_to: msg_id,
                        echo: echo.clone(),
                    });
                    serde_json::to_writer(&mut output, &reply).unwrap();
                    writeln!(&mut output).unwrap();
                }
                messages::Body::EchoOk {
                    msg_id: _,
                    in_reply_to: _,
                    echo: _,
                } => todo!(),
            };
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
