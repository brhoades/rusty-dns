use crate::dns::*;

use nom::{bytes::complete::take, named, number::Endianness, u16};
// final and intermediate parser results
type PResult<I, O> = nom::IResult<I, O, failure::Error>;

pub fn message_from_bytes(input: &[u8]) -> PResult<&[u8], Message> {
    let mut m = Message::default();
    let (rem, header) = header_from_bytes(input)?;

    m.header = header;
    Ok((rem, m))
}

named!(inner_take_u16<u16>, u16!(Endianness::Little));

fn take_u16(input: &[u8]) -> PResult<&[u8], u16> {
    match inner_take_u16(input) {
        Ok(x) => PResult::Ok(x),
        Err(Incomplete(e)) => PResult::Err(failure::format_err!("error parsing u16: {}", e)),
    }
}

fn header_from_bytes(input: &[u8]) -> PResult<&[u8], Header> {
    let (rem, id) = take_u16(input)?;
    let mut h = Header::default();
    h.id = id;

    return Ok((rem, h));
}
