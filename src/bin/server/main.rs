use clap::Parser;
use log::{debug, error, trace, warn};
use tokio::net::{TcpListener, TcpStream};

mod connection;
mod resp_codec;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A simple redis server written in Rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to start the server on
    #[arg(short, long, default_value_t = 6379)]
    port: i32,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    pretty_env_logger::init_timed();
    let args = Args::parse();

    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port)).await?;

    trace!("server starting on port {}", args.port);

    loop {
        match listener.accept().await {
            Ok((socket, _addr)) => {
                tokio::spawn(async move {
                    connection::Connection::init(socket).await;
                });
            }
            Err(e) => error!("error accepting connection {:?}", e),
        }
    }
}
