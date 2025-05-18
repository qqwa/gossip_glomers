use std::io::{self, BufReader};

use gossip_glomers::{Server2, router::Router, workloads::init};

fn main() {
    let mut router: Router<()> = Router::default();
    init::insert_handlers(&mut router);

    let reader = BufReader::new(io::stdin());
    let mut server = Server2::new(reader, io::stdout(), router, ());
    server.serve();
}
