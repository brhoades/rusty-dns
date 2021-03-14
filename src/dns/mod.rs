pub mod parse;

// These structs are copied from trust dns proto
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Message {
    pub header: Header,
    pub queries: Vec<Query>,
    pub answers: Vec<Record>,
    name_servers: Vec<Record>,
    additionals: Vec<Record>,
    sig0: Vec<Record>,
    edns: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Default)]
pub struct Header {
    id: u16,
    // qr
    message_type: MessageType,
    // middle 11 bits of 2nd header u16; not shifted, | with type + rcode u16
    mid_rem: u16,
    rcode: u8, // u4, right hand aligned

    query_count: u16,
    answer_count: u16,
    name_server_count: u16,
    additional_count: u16,
}

/// Message types are either Query (also Update) or Response
#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum MessageType {
    /// Queries are Client requests, these are either Queries or Updates
    Query,
    /// Response message from the Server or upstream Resolver
    Response,
}

impl Default for MessageType {
    fn default() -> MessageType {
        MessageType::Query
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Query {
    name: Name,
    record_type: u16,
    dns_class: u16,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Name {
    FQDN(Vec<String>),
    DN(Vec<String>),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Record {
    name: Name,
    record_type: u16,
    dns_class: u16,
    ttl: std::time::Duration,
    // rdlength-len resposne data
    rdata: Vec<u8>,
}

pub fn parse_message(input: &[u8]) -> Result<Message, failure::Error> {
    match parse::message_from_bytes(input) {
        nom::IResult::Ok((_, m)) => Ok(m),
        nom::IResult::Err(e) => Err(failure::format_err!("error when parsing message: {}", e)),
    }
}

impl Message {
    pub fn into_bytes(self) -> Vec<u8> {
        let mut res = Vec::with_capacity(512);
        res.extend(&self.header.into_bytes());

        res
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.clone().into_bytes()
    }
}

impl Header {
    pub fn into_bytes(self) -> Vec<u8> {
        let mut res = Vec::with_capacity(64);
        res.extend_from_slice(&self.id.to_ne_bytes()[..]);

        res
    }
}
