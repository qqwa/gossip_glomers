use std::{
    io::{Cursor, Read, Write},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

use gossip_glomers::Server;

pub struct TestServer {
    // input_raw_msgs: Vec<String>,
    // output_raw_msgs: Vec<String>,
    input_sender: Option<Sender<String>>,
    output_receiver: Receiver<String>,
    handle: JoinHandle<()>,
}

impl TestServer {
    pub fn new() -> TestServer {
        let (input_sender, input_receiver) = ReceiverRead::new();
        let (output_sender, output_receiver) = SenderWrite::new();

        let handle = thread::spawn(move || {
            let mut server = Server::new();
            let mut input = input_receiver;
            let mut output = output_sender;
            server.serve(&mut input, &mut output, &mut Vec::new());
        });

        TestServer {
            input_sender: Some(input_sender),
            output_receiver,
            handle,
        }
    }

    pub fn send_str(mut self, raw_msg: &str) -> Self {
        let msg = raw_msg.to_string() + "\n";
        let sender = self.input_sender.expect("Input closed");
        sender.send(msg).expect("Receiver is disconnected...");
        self.input_sender = Some(sender);
        self
    }

    pub fn expect_raw_message(self, want: &str) -> Self {
        let got = self
            .output_receiver
            .recv()
            .expect("Sender is disconnected...");
        assert_eq!(want, got);
        self
    }

    /// Not sure when this is useful, as it will happen automatically when
    /// `TestServer` gets dropped
    pub fn close(mut self) -> Self {
        self.input_sender = None;
        self
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
            let rem = self.buffer.split_off(index);
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
