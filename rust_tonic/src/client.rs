use echo::echoer_client::EchoerClient;
use echo::EchoRequest;

use clap::Parser;
use tonic::transport::Channel;

pub mod echo {
    tonic::include_proto!("echo");
}

fn size_parser(s: &str) -> Result<usize, anyhow::Error> {
    parse_size::Config::new()
        .with_binary()
        .parse_size(s)
        .map(|x| x as usize)
        .map_err(|e| anyhow::anyhow!("failed to parse {}: {:?}", s, e))
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

    #[arg(short, long, default_value_t = 1000)]
    reps: usize,

    #[arg(short = 's', long, default_value_t = 1, value_parser = size_parser)]
    message_size: usize,
}

async fn do_run(
    mut client: EchoerClient<Channel>,
    request: tonic::Request<EchoRequest>,
) -> anyhow::Result<usize> {
    let start = tokio::time::Instant::now();
    let _reply = client.echo(request).await?;
    Ok(start.elapsed().as_micros() as usize)
}

async fn run(args: Args) -> anyhow::Result<()> {
    let client = EchoerClient::connect(format!("http://{}:{}", args.host, args.port)).await?;
    tracing::info!("connected @ {}:{}", args.host, args.port);
    let request = EchoRequest {
        msg: vec![42u8; args.message_size].into(),
    };

    let futs = (0..args.reps)
        .into_iter()
        .map(|_| do_run(client.clone(), tonic::Request::new(request.clone())))
        .collect::<Vec<_>>();

    let results = futures::future::join_all(futs)
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;

    println!(
        "{}",
        results
            .into_iter()
            .map(|x| format!("{}us", x))
            .collect::<Vec<_>>()
            .join("\n")
    );
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
