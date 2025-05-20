use std::io::{self, BufReader};

use gossip_glomers::{Server, router::Router, workloads::init};

fn main() {
    let router: Router<()> = init::create_router();

    let reader = BufReader::new(io::stdin());
    let mut server = Server::new(reader, io::stdout(), router, ());
    server.serve();
}
