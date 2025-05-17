use std::io::{BufRead, BufReader, Read, Write};

use messages::{
    Broadcast, BroadcastOk, Echo, EchoOk, Generate, GenerateOk, Init, InitOk, Message, ReadOk,
    Topology, TopologyOk,
};
use serde_json::Value;
use uuid::Uuid;

pub mod messages;
pub struct Server {
    messages: Vec<Value>,
    me: Option<String>,
    neighbors: Vec<String>,
    input: Option<Box<dyn Read + Send + 'static>>,
    output: Box<dyn Write + Send + 'static>,
    log: Box<dyn Write + Send + 'static>,
}

impl Server {
    pub fn new<R: Read + Send + 'static, W1: Write + Send + 'static, W2: Write + Send + 'static>(
        input: R,
        output: W1,
        log: W2,
    ) -> Server {
        Server {
            messages: Vec::new(),
            me: None,
            neighbors: Vec::new(),
            input: Some(Box::new(input)),
            output: Box::new(output),
            log: Box::new(log),
        }
    }

    pub fn serve(&mut self) {
        let input = BufReader::new(self.input.take().unwrap());
        for line in input.lines() {
            let Ok(line) = line else {
                continue;
            };
            let Ok(message) = serde_json::from_str::<Message>(&line) else {
                continue;
            };
            match message.body {
                messages::Body::Init(Init {
                    msg_id,
                    ref node_id,
                    node_ids: _,
                }) => {
                    self.me = Some(node_id.clone());
                    let reply = message.create_response(messages::Body::InitOk(InitOk {
                        in_reply_to: msg_id,
                    }));
                    send_message(&mut self.output, &reply);
                }
                messages::Body::InitOk(InitOk { in_reply_to: _ }) => todo!(),
                messages::Body::Echo(Echo { msg_id, ref echo }) => {
                    let reply = message.create_response(messages::Body::EchoOk(EchoOk {
                        msg_id: None,
                        in_reply_to: msg_id,
                        echo: echo.clone(),
                    }));
                    send_message(&mut self.output, &reply);
                }
                messages::Body::EchoOk(EchoOk {
                    msg_id: _,
                    in_reply_to: _,
                    echo: _,
                }) => todo!(),
                messages::Body::Generate(Generate { msg_id }) => {
                    let reply = message.create_response(messages::Body::GenerateOk(GenerateOk {
                        in_reply_to: msg_id,
                        msg_id: None,
                        id: Uuid::new_v4().to_string(),
                    }));
                    send_message(&mut self.output, &reply);
                }
                messages::Body::GenerateOk(GenerateOk {
                    in_reply_to: _,
                    msg_id: _,
                    id: _,
                }) => todo!(),
                messages::Body::Broadcast(Broadcast {
                    message: ref msg,
                    msg_id,
                }) => {
                    let saved_message = self.save_message(msg);
                    if saved_message {
                        self.broadcast_value(msg.clone(), &message.src);
                    }
                    let reply = message.create_response(messages::Body::BroadcastOk(BroadcastOk {
                        in_reply_to: msg_id,
                        msg_id: None,
                    }));
                    send_message(&mut self.output, &reply);
                }
                messages::Body::BroadcastOk(BroadcastOk {
                    in_reply_to: _,
                    msg_id: _,
                }) => {
                    // TODO: have msg_ids for broadcast send and clear them when receiving ok
                }
                messages::Body::Read(messages::Read { msg_id }) => {
                    let reply = message.create_response(messages::Body::ReadOk(ReadOk {
                        in_reply_to: msg_id,
                        msg_id: None,
                        messages: self.messages.clone(),
                    }));
                    send_message(&mut self.output, &reply);
                }
                messages::Body::ReadOk(ReadOk {
                    in_reply_to: _,
                    msg_id: _,
                    messages: _,
                }) => todo!(),
                messages::Body::Topology(Topology {
                    msg_id,
                    ref topology,
                }) => {
                    writeln!(self.log, "Topology: {:#?}", topology).unwrap();
                    self.neighbors = topology
                        .get(self.me.as_ref().expect("Did not receive init message"))
                        .expect("topology did not include me")
                        .clone();
                    let reply = message.create_response(messages::Body::TopologyOk(TopologyOk {
                        in_reply_to: msg_id,
                        msg_id: None,
                    }));
                    send_message(&mut self.output, &reply);
                }
                messages::Body::TopologyOk(TopologyOk {
                    in_reply_to: _,
                    msg_id: _,
                }) => todo!(),
            };
        }
    }

    fn broadcast_value(&mut self, value: Value, src: &str) {
        let mut msg = Message {
            src: self.me.clone().unwrap(),
            dest: "".to_string(),
            body: messages::Body::Broadcast(Broadcast {
                message: value,
                msg_id: 1,
            }),
        };
        for node in &self.neighbors {
            if node != src {
                msg.dest = node.clone();
                send_message(&mut self.output, &msg);
            }
        }
    }

    /// Returns `true` if we haven't seen the message before
    fn save_message(&mut self, value: &Value) -> bool {
        if !self.messages.contains(value) {
            self.messages.push(value.clone());
            return true;
        }
        false
    }
}

fn send_message(mut output: &mut dyn Write, message: &Message) {
    serde_json::to_writer(&mut output, &message).unwrap();
    writeln!(&mut output).unwrap();
    output.flush().unwrap();
}
