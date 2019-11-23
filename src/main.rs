// #![deny(warnings, rust_2018_idioms)]
mod server;
use futures::future::lazy;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::run(lazy(|| {
        server::start_ws_server()
    }));

    Ok(())
}
