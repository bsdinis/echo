use echo::echoer_server::{Echoer, EchoerServer};
use echo::{EchoReply, EchoRequest};

use clap::Parser;
use tonic::{transport::Server, Request, Response, Status};

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

async fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", args.host, args.port).parse()?;
    let echoer = MyEchoer::default();

    tracing::info!("preparing to serve @ {}:{}", args.host, args.port);
    Server::builder()
        .add_service(EchoerServer::new(echoer))
        .serve(addr)
        .await?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .finish();
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
