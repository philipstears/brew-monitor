mod bluetooth_discovery;
pub use bluetooth_discovery::*;

mod grainfather_client;
pub use grainfather_client::*;

mod server;
pub use server::*;

pub fn main() {
    Server::new().run()
}
