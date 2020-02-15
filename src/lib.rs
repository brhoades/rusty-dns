use failure::{format_err, Error};
use std::env;
use std::str::FromStr;

use tokio::net::UdpSocket;
use tokio::runtime::Runtime;

use trust_dns_client::client::{AsyncClient, ClientHandle};
use trust_dns_client::rr::{DNSClass, Name, RData, RecordType};
use trust_dns_client::udp::UdpClientStream;

pub async fn bind<A: std::net::ToSocketAddrs + std::fmt::Debug>(
    server: A,
    remote: A,
) -> Result<(), failure::Error> {
    let serveraddr = server
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| format_err!("error parsing bind addr {:?}", remote))?;
    let mut socket = UdpSocket::bind(serveraddr).await?;
    let remoteaddr = remote
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| format_err!("error parsing remote {:?}", remote))?;
    println!("Listening on {}", socket.local_addr()?);

    loop {
        let mut buf = vec![0u8; 512];
        let (recv, peer) = socket.recv_from(&mut buf).await?;
        let resp = match trust_dns_proto::op::message::Message::from_vec(&buf[..recv]) {
            Ok(m) => dns_forward(&remoteaddr, m.to_vec()?).await?,
            Err(e) => {
                println!("error: {}", e);
                trust_dns_proto::op::message::Message::error_msg(
                    0,
                    trust_dns_proto::op::op_code::OpCode::Query,
                    trust_dns_proto::op::response_code::ResponseCode::ServFail,
                )
                .to_vec()
                .expect("created invalid error response")
            }
        };
        socket.send_to(&resp[..], &peer).await?;
    }
}

pub async fn dns_forward(
    remote: &std::net::SocketAddr,
    message: Vec<u8>,
) -> Result<Vec<u8>, failure::Error> {
    let mut socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| format_err!("error when binding local udp socket: {}", e))?;
    socket
        .send_to(&message, remote)
        .await
        .map_err(|e| format_err!("error when sending data to remote server {}: {}", remote, e))?;
    let mut data = vec![0u8; 512];
    socket.recv(&mut data).await?;

    Ok(data)
}

pub async fn resolve(
    r: &str,
    server: &str,
) -> Result<trust_dns_proto::xfer::dns_response::DnsResponse, Error> {
    let server_address = server.parse()?;
    let mut runtime = Runtime::new()?;

    // We need a connection, TCP and UDP are supported by DNS servers
    //   (tcp construction is slightly different as it needs a multiplexer)
    let stream = UdpClientStream::<UdpSocket>::new(server_address);

    // Create a new client, the bg is a background future which handles
    //   the multiplexing of the DNS requests to the server.
    //   the client is a handle to an unbounded queue for sending requests via the
    //   background. The background must be scheduled to run before the client can
    //   send any dns requests
    let client = AsyncClient::connect(stream);
    let (mut client, bg) = runtime.block_on(client)?;
    tokio::spawn(bg);

    // Create a query future
    match client
        .query(
            Name::from_str(format!("{}.", r).as_str())?,
            DNSClass::IN,
            RecordType::A,
        )
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => Err(format_err!("error: {}", e)),
    }

    /*
    // validate it's what we expected
    if let &RData::A(addr) = response.answers()[0].rdata() {
        Ok(Some(addr))
    } else {
        Ok(None)
    }
    */
}
