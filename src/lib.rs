use std::net::ToSocketAddrs;

use failure::{format_err, Error};
use log::*;

use tokio::net::UdpSocket;
use trust_dns_proto::op::Message;

mod cache;

pub async fn bind<A: ToSocketAddrs + std::fmt::Debug>(server: A, remote: A) -> Result<(), Error> {
    let serveraddr = server
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| format_err!("error parsing bind addr {:?}", remote))?;
    let mut socket = UdpSocket::bind(serveraddr).await?;
    let remoteaddr = remote
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| format_err!("error parsing remote {:?}", remote))?;
    info!("Listening on {}", socket.local_addr()?);

    let mut cache = cache::HostCache::new(1024);

    loop {
        let mut buf = vec![0u8; 512];
        let (recv, peer) = socket.recv_from(&mut buf).await?;
        let resp = match trust_dns_proto::op::message::Message::from_vec(&buf[..recv]) {
            Ok(m) => {
                let query = m.queries().get(0).unwrap();
                info!(
                    "-> ? {} {} {}",
                    query.query_type(),
                    query.query_class(),
                    query.name()
                );
                match cache.get_ip(query) {
                    Some(r) => r.clone().set_id(m.id()).to_vec()?,
                    None => {
                        let res = dns_forward(&remoteaddr, m.to_vec()?).await?;
                        cache.set_ip(query.clone(), res);
                        cache.get_ip(query).unwrap().to_vec()?
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
                trust_dns_proto::op::message::Message::error_msg(
                    0,
                    trust_dns_proto::op::op_code::OpCode::Query,
                    trust_dns_proto::op::response_code::ResponseCode::ServFail,
                )
                .to_vec()?
            }
        };

        socket.send_to(&resp[..], &peer).await?;
    }
}

pub async fn dns_forward(
    remote: &std::net::SocketAddr,
    message: Vec<u8>,
) -> Result<Message, Error> {
    let mut socket = UdpSocket::bind("0.0.0.0:0".to_socket_addrs()?.next().unwrap())
        .await
        .map_err(|e| format_err!("error when binding local udp socket: {}", e))?;
    socket
        .send_to(&message, remote)
        .await
        .map_err(|e| format_err!("error when sending data to remote server {}: {}", remote, e))?;
    let mut data = vec![0u8; 512];
    socket.recv(&mut data).await?;

    Ok(Message::from_vec(&data)?)
}
