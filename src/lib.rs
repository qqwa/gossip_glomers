use std::io::{BufRead, BufReader, Read, Write};

use messages::Message;
use serde_json::Value;
use uuid::Uuid;

pub mod messages;
pub struct Server {
    messages: Vec<Value>,
    me: Option<String>,
    neighbors: Vec<String>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            messages: Vec::new(),
            me: None,
            neighbors: Vec::new(),
        }
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
            // writeln!(&mut _log, "{}", line).unwrap();
            match message.body {
                messages::Body::Init {
                    msg_id,
                    ref node_id,
                    node_ids: _,
                } => {
                    self.me = Some(node_id.clone());
                    let reply = message.create_response(messages::Body::InitOk {
                        in_reply_to: msg_id,
                    });
                    send_message(&mut output, &reply);
                    // output.flush().unwrap();
                }
                messages::Body::InitOk { in_reply_to: _ } => todo!(),
                messages::Body::Echo { msg_id, ref echo } => {
                    let reply = message.create_response(messages::Body::EchoOk {
                        msg_id: None,
                        in_reply_to: msg_id,
                        echo: echo.clone(),
                    });
                    send_message(&mut output, &reply);
                }
                messages::Body::EchoOk {
                    msg_id: _,
                    in_reply_to: _,
                    echo: _,
                } => todo!(),
                messages::Body::Generate { msg_id } => {
                    let reply = message.create_response(messages::Body::GenerateOk {
                        in_reply_to: msg_id,
                        msg_id: None,
                        id: Uuid::new_v4().to_string(),
                    });
                    send_message(&mut output, &reply);
                }
                messages::Body::GenerateOk {
                    in_reply_to: _,
                    msg_id: _,
                    id: _,
                } => todo!(),
                messages::Body::Broadcast {
                    message: ref msg,
                    msg_id,
                } => {
                    self.save_message(msg);
                    self.broadcast_value(output, msg.clone(), &message.src);
                    let reply = message.create_response(messages::Body::BroadcastOk {
                        in_reply_to: msg_id,
                        msg_id: None,
                    });
                    send_message(&mut output, &reply);
                }
                messages::Body::BroadcastOk {
                    in_reply_to: _,
                    msg_id: _,
                } => {
                    // TODO: have msg_ids for broadcast send and clear them when receiving ok
                }
                messages::Body::Read { msg_id } => {
                    let reply = message.create_response(messages::Body::ReadOk {
                        in_reply_to: msg_id,
                        msg_id: None,
                        messages: self.messages.clone(),
                    });
                    send_message(&mut output, &reply);
                }
                messages::Body::ReadOk {
                    in_reply_to: _,
                    msg_id: _,
                    messages: _,
                } => todo!(),
                messages::Body::Topology {
                    msg_id,
                    ref topology,
                } => {
                    self.neighbors = topology
                        .get(self.me.as_ref().expect("Did not receive init message"))
                        .expect("topology did not include me")
                        .clone();
                    let reply = message.create_response(messages::Body::TopologyOk {
                        in_reply_to: msg_id,
                        msg_id: None,
                    });
                    send_message(&mut output, &reply);
                }
                messages::Body::TopologyOk {
                    in_reply_to: _,
                    msg_id: _,
                } => todo!(),
            };
        }
    }

    fn broadcast_value(&mut self, output: &mut dyn Write, value: Value, src: &str) {
        let mut msg = Message {
            src: self.me.clone().unwrap(),
            dest: "".to_string(),
            body: messages::Body::Broadcast {
                message: value,
                msg_id: 1,
            },
        };
        for node in &self.neighbors {
            if node != src {
                msg.dest = node.clone();
                send_message(output, &msg);
            }
        }
    }

    fn save_message(&mut self, value: &Value) {
        if !self.messages.contains(value) {
            self.messages.push(value.clone());
        }
    }
}

fn send_message(mut output: &mut dyn Write, message: &Message) {
    serde_json::to_writer(&mut output, &message).unwrap();
    writeln!(&mut output).unwrap();
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
