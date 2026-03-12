use std::io;

use crate::resp_codec::RespCodec;
use futures::{SinkExt, StreamExt};
use log::{debug, error, trace, warn};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct Connection {}

impl Connection {
    pub async fn init(socket: TcpStream) {
        let mut frame = Framed::new(socket, RespCodec {});

        loop {
            match frame.next().await {
                Some(Ok(command)) => trace!("got command {:?}", command),
                Some(Err(e)) => error!("got error {:?}", e),
                None => {
                    trace!("socket went away goodbye");
                    return;
                }
            }
        }
    }
}
