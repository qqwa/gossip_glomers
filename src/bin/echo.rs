use std::io::{self, BufReader};

use gossip_glomers::{
    Server,
    router::Router,
    workloads::{echo::insert_echo_handlers, init},
};

fn main() {
    let mut router: Router<()> = init::create_router();
    insert_echo_handlers(&mut router);

    let reader = BufReader::new(io::stdin());
    let mut server = Server::new(reader, io::stdout(), router, ());
    server.serve();
}
