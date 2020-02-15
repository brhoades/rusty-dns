use tokio::net::UdpSocket;
use tokio::prelude::*;

pub async fn bind(server: &str) -> std::io::Result<()> {
    let mut socket = UdpSocket::bind(server).await?;
    println!("Listening on {}", socket.local_addr()?);

    let mut buf = vec![0u8; 512];

    loop {
        let (recv, peer) = socket.recv_from(&mut buf).await?;
        let sent = socket.send_to(&buf[..recv], &peer).await?;
    }
}

pub async fn resolve(r: &str, server: &str) -> Result<Option<std::net::Ipv4Addr>, failure::Error> {
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    use tokio::runtime::Runtime;
    use trust_dns_client::client::{AsyncClient, Client, ClientHandle};
    use trust_dns_client::op::ResponseCode;
    use trust_dns_client::rr::rdata::key::KEY;
    use trust_dns_client::rr::{DNSClass, Name, RData, Record, RecordType};
    use trust_dns_client::udp::UdpClientStream;
    let mut runtime = Runtime::new()?;

    // We need a connection, TCP and UDP are supported by DNS servers
    //   (tcp construction is slightly different as it needs a multiplexer)
    let stream = UdpClientStream::<UdpSocket>::new(([8, 8, 8, 8], 53).into());

    // Create a new client, the bg is a background future which handles
    //   the multiplexing of the DNS requests to the server.
    //   the client is a handle to an unbounded queue for sending requests via the
    //   background. The background must be scheduled to run before the client can
    //   send any dns requests
    let client = AsyncClient::connect(stream);
    let (mut client, bg) = runtime.block_on(client)?;
    runtime.spawn(bg);

    // Create a query future
    let response = client
        .query(
            Name::from_str(format!("{}.", r).as_str())?,
            DNSClass::IN,
            RecordType::A,
        )
        .await?;

    // validate it's what we expected
    if let &RData::A(addr) = response.answers()[0].rdata() {
        Ok(Some(addr))
    } else {
        Ok(None)
    }
}
