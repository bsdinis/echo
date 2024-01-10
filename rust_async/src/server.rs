use clap::Parser;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use anyhow::Context;

const BUFFER_SIZE: usize = 1 << 16;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(default_value = "[::1]")]
    host: String,

    #[arg(default_value_t = 9095, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,

    #[arg(short = 'j', long)]
    n_cores: Option<usize>,
}

async fn handle_client(mut stream: TcpStream) -> anyhow::Result<()> {
    let mut data = [0 as u8; BUFFER_SIZE];
    let mut size_buffer = [0; 8];
    stream.read_exact(&mut size_buffer).await?;
    let message_size = usize::from_be_bytes(size_buffer);

    loop {
        let mut to_read = message_size;
        while to_read > 0 {
            match stream.read(&mut data).await {
                Ok(size) => {
                    stream
                        .write(&data[0..size])
                        .await
                        .context("failed to echo")?;
                    to_read -= size;
                }
                Err(e) => {
                    let err = Err(e).context(format!(
                        "An error occurred, terminating connection with {}",
                        stream.peer_addr().unwrap()
                    ));
                    stream.shutdown().await.unwrap();
                    return err;
                }
            }
        }
    }
}

async fn run(args: Args) -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", args.host, args.port)).await?;
    tracing::info!("server listening on {}:{}", args.host, args.port);

    loop {
        match listener.accept().await {
            Ok((stream, socket_addr)) => {
                tracing::info!("accepted new connection: {}", socket_addr);
                tokio::spawn(async move {
                    // connection succeeded
                    if let Err(e) = handle_client(stream).await {
                        tracing::warn!("failed to handle connection from {}: {:?}", socket_addr, e);
                    }
                });
            }
            Err(e) => {
                tracing::warn!("failed to accept connection: {}", e);
                /* connection failed */
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let rt = if let Some(n_cores) = args.n_cores {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(n_cores)
            .build()
            .unwrap()
    } else {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    };

    rt.block_on(run(args))
}
