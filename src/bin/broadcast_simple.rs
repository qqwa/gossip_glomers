use std::io::{self, BufReader};

use gossip_glomers::{
    Server2,
    router::Router,
    workloads::{broadcast::insert_broadcast_simple_handlers, init},
};

fn main() {
    let mut router: Router<_> = init::create_router();
    insert_broadcast_simple_handlers(&mut router);

    let reader = BufReader::new(io::stdin());
    let mut server = Server2::new(reader, io::stdout(), router, Default::default());
    server.serve();
}
