use std::env;
use rusty_dns::bind;

use lru_cache::LruCache;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    pretty_env_logger::init();

    let bind_addr: std::net::SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:53530".into())
        .parse()?;
    let remote_addr: std::net::SocketAddr = env::args()
        .nth(2)
        .unwrap_or_else(|| "1.1.1.1:53".into())
        .parse()?;
    let cache = LruCache::new(1024);
    bind(&bind_addr, &remote_addr, cache).await
}
