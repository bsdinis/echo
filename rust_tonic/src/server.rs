use std::net::{SocketAddr, ToSocketAddrs};

use echo::echoer_server::{Echoer, EchoerServer};
use echo::{EchoReply, EchoRequest};

use clap::Parser;
use tonic::{transport::Server, Request, Response, Status};

use anyhow::Context;

pub mod echo {
    tonic::include_proto!("echo");
}

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(default_value = "[::1]")]
    host: String,

    #[arg(default_value_t = 9091, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,

    #[arg(short = 'j', long)]
    n_cores: Option<usize>,
}

#[derive(Debug, Default)]
pub struct MyEchoer {}

#[tonic::async_trait]
impl Echoer for MyEchoer {
    async fn echo(&self, request: Request<EchoRequest>) -> Result<Response<EchoReply>, Status> {
        tracing::info!(
            "handling request from {}",
            request
                .remote_addr()
                .map(|x| format!("{}", x))
                .unwrap_or("unknown".to_string())
        );
        let reply = echo::EchoReply {
            msg: request.into_inner().msg,
        };

        Ok(Response::new(reply))
    }
}

async fn run(args: Args) -> anyhow::Result<()> {
    let addr: SocketAddr = (args.host.as_str(), args.port)
        .to_socket_addrs()
        .context("failed to parse")?
        .next()
        .ok_or_else(|| anyhow::anyhow!("no socket addrs"))?;
    let echoer = MyEchoer::default();

    tracing::info!("preparing to serve @ {}:{}", args.host, args.port);
    Server::builder()
        .add_service(EchoerServer::new(echoer))
        .serve(addr)
        .await?;

    Ok(())
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
