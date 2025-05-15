use std::io::{stderr, stdin, stdout};

use gossip_glomers::Server;

fn main() {
    let mut server = Server::new(stdin(), stdout(), stderr());
    server.serve();
}
