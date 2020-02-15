use criterion::{criterion_group, criterion_main, Criterion};
use std::str::FromStr;
use tokio::runtime::Runtime;

use trust_dns_client::client::{Client, SyncClient};
use trust_dns_client::op::DnsResponse;
use trust_dns_client::rr::{DNSClass, Name, RData, Record, RecordType};
use trust_dns_client::udp::UdpClientConnection;

// sync resolver
fn resolve(r: &str, server: &str) -> Result<Option<std::net::Ipv4Addr>, failure::Error> {
    let address = server.parse()?;
    let conn = UdpClientConnection::new(address)?;

    // and then create the Client
    let client = SyncClient::new(conn);

    // Specify the name, note the final '.' which specifies it's an FQDN
    let name = Name::from_str(r)?;

    // NOTE: see 'Setup a connection' example above
    // Send the query and get a message response, see RecordType for all supported options
    let response: DnsResponse = client.query(&name, DNSClass::IN, RecordType::A)?;

    // Messages are the packets sent between client and server in DNS.
    //  there are many fields to a Message, DnsResponse can be dereferenced into
    //  a Message. It's beyond the scope of these examples
    //  to explain all the details of a Message. See trust_dns_client::op::message::Message for more details.
    //  generally we will be interested in the Message::answers
    let answers: &[Record] = response.answers();

    if answers.len() == 0 {
        return Ok(None);
    }

    // Records are generic objects which can contain any data.
    //  In order to access it we need to first check what type of record it is
    //  In this case we are interested in A, IPv4 address
    if let &RData::A(ref ip) = answers[0].rdata() {
        return Ok(Some(*ip));
    }

    return Ok(None);
}

fn criterion_benchmark(c: &mut Criterion) {
    let server = "127.0.0.1:14582";
    let bg = rusty_dns::bind(server);
    let runtime = Runtime::new().unwrap();
    runtime.spawn(bg);

    c.bench_function("my dns", |b| {
        b.iter(|| match resolve("google.com", server) {
            Ok(_) => (),
            Err(e) => println!("ERROR: {}", e),
        })
    });
    c.bench_function("baseline: 1.1.1.1", |b| {
        b.iter(|| match resolve("google.com", "1.1.1.1:53") {
            Ok(_) => (),
            Err(e) => println!("ERROR: {}", e),
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
