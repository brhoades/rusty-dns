use rusty_dns::bind;
use tokio::prelude::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    bind("127.0.0.1:8080").await
}
