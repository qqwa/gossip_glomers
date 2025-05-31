use std::{
    fmt::Debug,
    io::{BufRead, Write},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use messages::{Body, Message};
use router::Router;

pub mod messages;
pub mod router;
pub mod workloads;

#[cfg(test)]
mod testing;

pub struct Server<U>
where
    U: Debug,
{
    router: Router<U>,
    user_data: U,
    maelstrom_data: Maelstrom,
    rx_input: Receiver<Message>,
    tx_output: Sender<Message>,
}

#[derive(Default, Debug)]
pub struct Maelstrom {
    node_id: String,
    counter: u64,
}

impl Maelstrom {
    pub fn create_message(&self, dest: &str, body: Body) -> Message {
        Message {
            src: self.node_id.clone(),
            dest: dest.to_string(),
            body,
        }
    }

    pub fn generate_id(&mut self) -> u64 {
        let id = self.counter;
        self.counter += 1;
        id
    }
}

impl<U> Server<U>
where
    U: Debug,
{
    pub fn new<R: BufRead + Send + 'static, W: Write + Send + 'static>(
        reader: R,
        writer: W,
        router: Router<U>,
        user_data: U,
    ) -> Self {
        let (tx_input, rx_input) = mpsc::channel();
        let (tx_output, rx_output) = mpsc::channel();
        Self::start_input_thread(reader, tx_input);
        Self::start_output_thread(writer, rx_output);
        Self {
            router,
            user_data,
            rx_input,
            tx_output,
            maelstrom_data: Maelstrom::default(),
        }
    }

    fn start_input_thread<R: BufRead + Send + 'static>(
        mut reader: R,
        tx_input: Sender<Message>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Err(_) => break,
                    Ok(_) => match serde_json::from_str::<Message>(&line) {
                        Ok(message) => tx_input
                            .send(message)
                            .expect("Channel closed, panic will end this thread"),
                        Err(err) => eprintln!("Could not parse message: {}", err),
                    },
                }
            }
        })
    }

    fn start_output_thread<W: Write + Send + 'static>(
        mut writer: W,
        rx_output: Receiver<Message>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            for message in rx_output {
                match serde_json::to_string(&message) {
                    Ok(text) => writeln!(writer, "{}", text).expect("Error writing to output"),
                    Err(err) => {
                        eprintln!("Could not convert {:?} to json string: {}", message, err)
                    }
                }
            }
        })
    }

    pub fn serve(&mut self) {
        loop {
            if let Ok(msg) = self.rx_input.recv_timeout(Duration::from_millis(50)) {
                self.router.handle(
                    msg,
                    &mut self.tx_output,
                    &mut self.maelstrom_data,
                    &mut self.user_data,
                );
            }
            self.router.tick(
                &mut self.tx_output,
                &mut self.maelstrom_data,
                &mut self.user_data,
            );
        }
    }
}
