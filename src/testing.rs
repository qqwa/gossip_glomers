use std::{
    io::{BufReader, Cursor, Read, Write},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use crate::{Server2, messages::Message, router::Router};

pub struct TestServer {
    output_raw_msgs: Vec<Message>,
    input_sender: Option<Sender<String>>,
    output_receiver: Receiver<String>,
}

impl TestServer {
    pub fn from_router<U: Default + 'static>(router: Router<U>) -> TestServer {
        let (input_sender, input_receiver) = ReceiverRead::new();
        let (output_sender, output_receiver) = SenderWrite::new();

        thread::spawn(move || {
            let reader = BufReader::new(input_receiver);
            let mut server = Server2::new(reader, output_sender, router, U::default());
            server.serve();
        });

        TestServer {
            output_raw_msgs: Vec::new(),
            input_sender: Some(input_sender),
            output_receiver,
        }
    }

    pub fn send_str(mut self, raw_msg: &str) -> Self {
        let msg = raw_msg.to_string() + "\n";
        let sender = self.input_sender.expect("Input closed");
        sender.send(msg).expect("Receiver is disconnected...");
        self.input_sender = Some(sender);
        self
    }

    pub fn assert_msg_received_default_timeout<F>(&mut self, predicate: F)
    where
        F: Fn(&Message) -> bool,
    {
        self.assert_msg_received_timeout(predicate, Duration::from_millis(10));
    }

    pub fn assert_msg_received_timeout<F>(&mut self, predicate: F, timeout: Duration)
    where
        F: Fn(&Message) -> bool,
    {
        for msg in &self.output_raw_msgs {
            if predicate(msg) {
                return;
            }
        }

        let start = Instant::now();
        while start.elapsed() < timeout {
            match self.output_receiver.recv_timeout(timeout - start.elapsed()) {
                Ok(msg_str) => {
                    let msg = parse_raw_message(msg_str);
                    let found = predicate(&msg);
                    self.output_raw_msgs.push(msg);
                    if found {
                        return;
                    }
                }
                Err(_) => {}
            }
        }

        panic!(
            "No matching message was cached or received before timeout buffer:\n{:#?}",
            self.output_raw_msgs
        );
    }
}

fn parse_raw_message(raw_msg: String) -> Message {
    match serde_json::from_str(&raw_msg) {
        Ok(msg) => msg,
        Err(err) => panic!("Could not parse message: {}", err),
    }
}
pub struct ReceiverRead {
    receiver: Receiver<String>,
    current_chunk: Cursor<Vec<u8>>,
}

impl ReceiverRead {
    pub fn new() -> (Sender<String>, Self) {
        let (sender, receiver) = mpsc::channel();
        (
            sender,
            ReceiverRead {
                receiver,
                current_chunk: Cursor::new(Vec::new()),
            },
        )
    }
}

impl Read for ReceiverRead {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.current_chunk.position() >= self.current_chunk.get_ref().len() as u64 {
            match self.receiver.recv() {
                Ok(msg) => self.current_chunk = Cursor::new(msg.into_bytes()),
                Err(_) => return Ok(0),
            }
        }
        self.current_chunk.read(buf)
    }
}

pub struct SenderWrite {
    sender: Sender<String>,
    buffer: Vec<u8>,
}

impl SenderWrite {
    pub fn new() -> (Self, Receiver<String>) {
        let (sender, receiver) = mpsc::channel();
        (
            SenderWrite {
                sender,
                buffer: vec![],
            },
            receiver,
        )
    }
}

impl Write for SenderWrite {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(&buf);
        let index = self.buffer.iter().position(|c| c == &b'\n');
        if let Some(index) = index {
            let rem = self.buffer.split_off(index + 1);
            self.buffer.pop(); // remove b'\n'
            self.sender
                .send(String::from_utf8(self.buffer.clone()).unwrap())
                .expect("Receiver disconnected...");
            self.buffer = rem;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
