pub mod cache;
pub mod dns;

use std::net::ToSocketAddrs;

use failure::{format_err, Error};
use log::*;

use cache::Cache;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

use dns::{Message, Query};

pub async fn bind<
    A: ToSocketAddrs + std::fmt::Debug,
    C: Cache<Key = dns::Query, Value = dns::Message>,
>(
    server: A,
    remote: A,
    cache: C,
) -> Result<(), Error> {
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

    let cache: Mutex<C> = Mutex::new(cache);

    loop {
        let mut buf = vec![0u8; 512];
        let (recv, peer) = socket.recv_from(&mut buf).await?;
        let msg: Vec<u8> = match dns::parse_message(&buf[..]) {
            Ok(m) => {
                let query: &Query = m.queries.get(0).unwrap();

                let mut c = cache.lock().await;
                let r = c.get(query);

                if let Some(answer) = r {
                    // msg.add_query(query.to_owned());
                    // TODO: remaining queries
                    answer.to_bytes()
                } else {
                    // drop the guard immediately
                    std::mem::drop(c);
                    let mut msg = dns_forward(&remoteaddr, m.to_bytes()).await?;
                    cache.lock().await.insert(query.clone(), msg.clone());
                    msg.into_bytes()
                }
            }
            Err(e) => (&[0, 0, 0]).to_vec(),
        };

        socket.send_to(&msg[..], &peer).await?;
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

    Ok(dns::parse_message(&data)?)
}
