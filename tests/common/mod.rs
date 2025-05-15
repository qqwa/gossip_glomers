use std::{
    io::{Cursor, Read, Write},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

use gossip_glomers::{Server, messages::Message};

pub struct TestServer {
    // input_raw_msgs: Vec<String>,
    output_raw_msgs: Vec<String>,
    input_sender: Option<Sender<String>>,
    output_receiver: Receiver<String>,
    log_receiver: Receiver<String>,
    handle: JoinHandle<()>,
}

pub struct ClosedServer {
    output_raw_msgs: Vec<String>,
}

impl ClosedServer {
    pub fn new(output_raw_msgs: Vec<String>) -> Self {
        Self { output_raw_msgs }
    }
    pub fn get_parsed_messages(&mut self) -> Vec<Message> {
        self.output_raw_msgs
            .iter()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }
}

impl TestServer {
    pub fn new() -> TestServer {
        let (input_sender, input_receiver) = ReceiverRead::new();
        let (output_sender, output_receiver) = SenderWrite::new();
        let (log_sender, log_receiver) = SenderWrite::new();

        let handle = thread::spawn(move || {
            let mut server = Server::new(input_receiver, output_sender, log_sender);
            server.serve();
        });

        TestServer {
            output_raw_msgs: Vec::new(),
            input_sender: Some(input_sender),
            output_receiver,
            log_receiver,
            handle,
        }
    }

    pub fn init() -> TestServer {
        TestServer::new()
        .send_str( r#"{"id":0,"src":"c0","dest":"n0","body":{"type":"init","node_id":"n0","node_ids":["n0"],"msg_id":1}}"#)
        .expect_raw_message(r#"{"src":"n0","dest":"c0","body":{"type":"init_ok","in_reply_to":1}}"#)
    }

    pub fn send_str(mut self, raw_msg: &str) -> Self {
        let msg = raw_msg.to_string() + "\n";
        let sender = self.input_sender.expect("Input closed");
        sender.send(msg).expect("Receiver is disconnected...");
        self.input_sender = Some(sender);
        self
    }

    pub fn expect_raw_message(mut self, want: &str) -> Self {
        let got = self.wait_get_message();
        assert_eq!(want, got);
        self
    }

    pub fn wait_get_message(&mut self) -> &String {
        let msg = self.output_receiver.recv().expect("Channel disconnected..");
        self.output_raw_msgs.push(msg);
        self.output_raw_msgs.last().unwrap()
    }

    /// Closes the input and collects all remaining messages from the server
    pub fn close(mut self) -> ClosedServer {
        self.input_sender = None;
        self.consume_all_messages();
        ClosedServer::new(self.output_raw_msgs)
    }

    fn consume_all_messages(&mut self) {
        loop {
            if let Ok(msg) = self.output_receiver.recv() {
                self.output_raw_msgs.push(msg);
            } else {
                return;
            }
        }
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
