use bytes::{Buf, Bytes, BytesMut};
use log::{trace, warn};
use std::{
    fmt,
    io::{self, Cursor},
    string::FromUtf8Error,
};
use tokio_util::codec::{Decoder, Encoder};

const R: u8 = b'\r';
const N: u8 = b'\n';

#[derive(Debug)]
pub enum RespFrame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<RespFrame>),
}

#[derive(Debug)]
pub enum Error {
    IncompleteFrame,
    Other(crate::Error),
}

pub struct RespCodec {}

impl Decoder for RespCodec {
    type Item = RespFrame;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() == 0 {
            return Ok(None);
        }

        let mut buf = Cursor::new(src);

        let data = Bytes::copy_from_slice(buf.get_ref());

        trace!("decoding buffer {:?}", String::from_utf8_lossy(&data));

        match parse(&mut buf) {
            Ok(frame) => {
                let pos = buf.position();
                buf.get_mut().advance(pos as usize);
                return Ok(Some(frame));
            }
            Err(e) => {
                warn!("error parsing {:?}", e);
                return Ok(None);
            }
        }
    }
}

impl Encoder<String> for RespCodec {
    type Error = Error;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src.into())
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Error {
        src.to_string().into()
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_src: FromUtf8Error) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl From<io::Error> for Error {
    fn from(src: io::Error) -> Error {
        format!("io error: {:?}", src).into()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IncompleteFrame => "waiting for more data".fmt(fmt),
            Error::Other(err) => err.fmt(fmt),
        }
    }
}

impl std::error::Error for Error {}

fn parse(buf: &mut Cursor<&mut BytesMut>) -> Result<RespFrame, Error> {
    match get_u8(buf)? {
        b'-' => {
            let line = get_line(buf)?.to_vec();

            let string = String::from_utf8(line)?;

            return Ok(RespFrame::Error(string));
        }
        b'+' => {
            let line = get_line(buf)?.to_vec();

            let string = String::from_utf8(line)?;

            return Ok(RespFrame::Simple(string));
        }
        b'$' => {
            if peek_u8(buf)? == b'-' {
                let line = get_line(buf)?;

                if line != b"-1" {
                    return Err("invalid frame".into());
                }

                return Ok(RespFrame::Null);
            }
            // get the length of the bulk string
            let len = get_num(buf)?;
            // get the current position of the cursor
            let pos = buf.position();
            // skip over the \r\n
            let n = (len + 2) as usize;

            if buf.remaining() < n {
                return Err(Error::IncompleteFrame);
            }

            // here we copy data from the cursor position to the position + length of the string
            let data = Bytes::copy_from_slice(&buf.get_ref()[pos as usize..(len + pos) as usize]);

            // advance cursor string length + \r\n
            skip(buf, n)?;

            trace!("got bulk string {:?}", String::from_utf8_lossy(&data));

            Ok(RespFrame::Bulk(data))
        }
        b':' => {
            let num = get_num(buf)?;

            return Ok(RespFrame::Integer(num));
        }
        b'*' => {
            let size = get_num(buf)?;
            let mut arr = Vec::with_capacity(size as usize);

            for _ in 0..size {
                arr.push(parse(buf)?);
            }

            trace!("returning cmd arr {:?}", arr);

            return Ok(RespFrame::Array(arr));
        }
        _ => unimplemented!("whoops"),
    }
}

fn get_num(buf: &mut Cursor<&mut BytesMut>) -> Result<u64, Error> {
    use atoi::atoi;

    let line = get_line(buf)?;

    atoi::<u64>(line).ok_or_else(|| "invalid number".into())
}

fn get_line<'a, 'b>(buf: &'a mut Cursor<&'b mut BytesMut>) -> Result<&'a [u8], Error> {
    let start = buf.position() as usize;

    let end = buf.get_ref().len() - 1;

    for i in start..end {
        if buf.get_ref()[i] == R && buf.get_ref()[i + 1] == N {
            buf.set_position((i + 2) as u64);

            return Ok(&buf.get_ref()[start..i]);
        }
    }

    Err("invalid line".into())
}

fn skip(buf: &mut Cursor<&mut BytesMut>, n: usize) -> Result<(), Error> {
    if buf.remaining() < n {
        return Err("Buf empty".into());
    }

    buf.advance(n);
    Ok(())
}

fn get_u8(buf: &mut Cursor<&mut BytesMut>) -> Result<u8, Error> {
    if buf.remaining() == 0 {
        return Err("Buf empty".into());
    }

    Ok(buf.get_u8())
}

fn peek_u8(buf: &mut Cursor<&mut BytesMut>) -> Result<u8, Error> {
    if buf.remaining() == 0 {
        return Err("Buf empty".into());
    }

    Ok(buf.get_ref()[buf.position() as usize])
}
