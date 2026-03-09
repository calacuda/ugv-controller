use crate::server::server;

pub mod server;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    server()
}
