use std::env;
use std::str::FromStr;

use rusty_dns::bind;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    let bind_addr: std::net::SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:53530".into())
        .parse()?;
    let remote_addr: std::net::SocketAddr = env::args()
        .nth(2)
        .unwrap_or_else(|| "1.1.1.1:53".into())
        .parse()?;
    bind(&bind_addr, &remote_addr).await
}
