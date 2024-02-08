use anyhow::Context;
use chrono::{Local, NaiveDateTime, NaiveTime};
use clap::Parser;
use echo::echoer_client::EchoerClient;
use echo::EchoRequest;
use tonic::transport::Channel;

pub mod echo {
    tonic::include_proto!("echo");
}

fn size_parser(s: &str) -> anyhow::Result<usize> {
    parse_size::Config::new()
        .with_binary()
        .parse_size(s)
        .map(|x| x as usize)
        .map_err(|e| anyhow::anyhow!("failed to parse {}: {:?}", s, e))
}

fn time_parser(s: &str) -> anyhow::Result<NaiveDateTime> {
    let today = Local::now().date_naive();
    let time =
        NaiveTime::parse_from_str(s, "%H:%M:%S").context("failed to parse start timestamp")?;
    Ok(NaiveDateTime::new(today, time))
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum ClientType {
    Bursty,
    ControlledBursty,
    Closed,
}

#[derive(Parser, Clone)]
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

    #[arg(long, value_parser = time_parser)]
    start: Option<NaiveDateTime>,

    #[arg(short, long)]
    client_type: ClientType,
}

async fn do_run(
    mut client: EchoerClient<Channel>,
    request: tonic::Request<EchoRequest>,
) -> anyhow::Result<()> {
    let start = tokio::time::Instant::now();
    let _reply = client.echo(request).await?;
    println!("{:.3}", start.elapsed().as_secs_f64() * 1_000_000f64);
    Ok(())
}

async fn closed_client(args: Args, reps: usize) -> anyhow::Result<()> {
    let client = EchoerClient::connect(format!("http://{}:{}", args.host, args.port)).await?;
    tracing::info!("connected @ {}:{}", args.host, args.port);
    let request = EchoRequest {
        msg: vec![42u8; args.message_size].into(),
    };

    for _ in 0..reps {
        do_run(client.clone(), tonic::Request::new(request.clone())).await?;
    }

    Ok(())
}

async fn run_closed(args: Args) -> anyhow::Result<()> {
    let paralellism = args.n_cores.unwrap_or_else(|| num_cpus::get());
    let start = tokio::time::Instant::now();
    let runners = (0..paralellism)
        .into_iter()
        .map(|idx| {
            if idx == 0 {
                args.reps / paralellism + args.reps % paralellism
            } else {
                args.reps / paralellism
            }
        })
        .map(|reps| closed_client(args.clone(), reps))
        .collect::<Vec<_>>();
    futures::future::join_all(runners)
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;
    let elapsed = start.elapsed();

    println!("Elapsed: {:.9}", elapsed.as_secs_f64());
    println!("Message Size: {}", args.message_size);
    Ok(())
}

async fn run_bursty(args: Args) -> anyhow::Result<()> {
    let client = EchoerClient::connect(format!("http://{}:{}", args.host, args.port)).await?;
    tracing::info!("connected @ {}:{}", args.host, args.port);
    let request = EchoRequest {
        msg: vec![42u8; args.message_size].into(),
    };

    let start = tokio::time::Instant::now();
    let futs = (0..args.reps)
        .into_iter()
        .map(|_| do_run(client.clone(), tonic::Request::new(request.clone())))
        .collect::<Vec<_>>();

    futures::future::join_all(futs)
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;
    let elapsed = start.elapsed();

    println!("Elapsed: {:.9}", elapsed.as_secs_f64());
    println!("Message Size: {}", args.message_size);
    Ok(())
}

async fn run_controlled_bursty(args: Args) -> anyhow::Result<()> {
    let client = EchoerClient::connect(format!("http://{}:{}", args.host, args.port)).await?;
    tracing::info!("connected @ {}:{}", args.host, args.port);
    let request = EchoRequest {
        msg: vec![42u8; args.message_size].into(),
    };

    let mut rem = args.reps;
    let paralellism = args.n_cores.unwrap_or_else(|| num_cpus::get());
    let start = tokio::time::Instant::now();
    while rem > 0 {
        let burst = std::cmp::min(rem, paralellism);
        let futs = (0..burst)
            .into_iter()
            .map(|_| do_run(client.clone(), tonic::Request::new(request.clone())))
            .collect::<Vec<_>>();

        futures::future::join_all(futs)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?;

        rem -= burst;
    }
    let elapsed = start.elapsed();

    println!("Elapsed: {:.9}", elapsed.as_secs_f64());
    println!("Message Size: {}", args.message_size);
    Ok(())
}

async fn run(args: Args) -> anyhow::Result<()> {
    match args.client_type {
        ClientType::Bursty => run_bursty(args).await,
        ClientType::ControlledBursty => run_controlled_bursty(args).await,
        ClientType::Closed => run_closed(args).await,
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
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

    if let Some(start) = args.start {
        let now_ts = Local::now().timestamp_nanos_opt().ok_or(anyhow::anyhow!("an i64 can represent stuff until 2262. if you are still using this code in 2262 first of all, thanks i guess; second, i don't really care, probably fix this"))?;
        let start_ts = start.timestamp_nanos_opt().ok_or(anyhow::anyhow!("an i64 can represent stuff until 2262. if you are still using this code in 2262 first of all, thanks i guess; second, i don't really care, probably fix this"))?;
        if now_ts < start_ts {
            std::thread::sleep(std::time::Duration::from_nanos((start_ts - now_ts) as u64));
        }
    }
    rt.block_on(run(args))
}
