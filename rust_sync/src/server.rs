use clap::Parser;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

use anyhow::Context;

const BUFFER_SIZE: usize = 1 << 16;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(default_value = "[::1]")]
    host: String,

    #[arg(default_value_t = 9094, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,
}

fn handle_client(mut stream: TcpStream) -> anyhow::Result<()> {
    let mut data = [0 as u8; BUFFER_SIZE];
    let mut size_buffer = [0; 8];
    stream.read_exact(&mut size_buffer)?;
    let message_size = usize::from_be_bytes(size_buffer);

    loop {
        let mut to_read = message_size;
        while to_read > 0 {
            match stream.read(&mut data) {
                Ok(size) => {
                    stream.write(&data[0..size]).context("failed to echo")?;
                    to_read -= size;
                }
                Err(e) => {
                    let err = Err(e).context(format!(
                        "An error occurred, terminating connection with {}",
                        stream.peer_addr().unwrap()
                    ));
                    stream.shutdown(Shutdown::Both).unwrap();
                    return err;
                }
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let listener = TcpListener::bind(format!("{}:{}", args.host, args.port))?;
    tracing::info!("server listening on {}:{}", args.host, args.port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                tracing::info!("accepted new connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || {
                    let peer_addr = stream.peer_addr().unwrap();
                    // connection succeeded
                    if let Err(e) = handle_client(stream) {
                        tracing::warn!("failed to handle connection from {}: {:?}", peer_addr, e);
                    }
                });
            }
            Err(e) => {
                tracing::warn!("failed to accept connection: {}", e);
                /* connection failed */
            }
        }
    }

    drop(listener);
    Ok(())
}
