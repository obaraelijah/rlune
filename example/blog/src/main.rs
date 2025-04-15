use std::net::SocketAddr;
use std::str::FromStr;

use rlune::contrib::TracingModule;
use rlune::Rlune;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rlune = Rlune::init();

    rlune.register_module(TracingModule::default());

    rlune.start(SocketAddr::from_str("127.0.0.1:8080")?).await?;

    Ok(())
}
