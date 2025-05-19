use std::io::{self, BufReader};

use gossip_glomers::{
    Server2,
    router::Router,
    workloads::{init, unique_id::insert_unique_id_handlers},
};

fn main() {
    let mut router: Router<()> = init::create_router();
    insert_unique_id_handlers(&mut router);

    let reader = BufReader::new(io::stdin());
    let mut server = Server2::new(reader, io::stdout(), router, ());
    server.serve();
}
