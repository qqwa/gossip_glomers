use std::io::{stderr, stdin, stdout};

use gossip_glomers::Server;

fn main() {
    let mut server = Server::new();
    server.serve(
        &mut stdin().lock(),
        &mut stdout().lock(),
        &mut stderr().lock(),
    );
}
