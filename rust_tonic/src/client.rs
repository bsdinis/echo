use anyhow::Context;
use chrono::{Local, NaiveDateTime, NaiveTime};
use clap::Parser;
use echo::echoer_client::EchoerClient;
use echo::EchoRequest;
use tokio::time::Duration;
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

fn duration_parser(s: &str) -> anyhow::Result<u64> {
    humantime::parse_duration(s)
        .map(|s| s.as_secs())
        .context("failed to parse duration")
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum ClientType {
    Bursty,
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

    #[arg(short, long, default_value = "60s", value_parser = duration_parser)]
    duration: u64,

    #[arg(short, long, default_value = "10s", value_parser = duration_parser)]
    warmup: u64,

    #[arg(short, long, default_value_t = 1, value_parser = size_parser)]
    message_size: usize,

    #[arg(short, long, value_parser = time_parser)]
    start: Option<NaiveDateTime>,

    #[arg(short, long)]
    client_type: ClientType,
}
impl Args {
    fn duration(&self) -> Duration {
        return Duration::from_secs(self.duration);
    }
    fn warmup(&self) -> Duration {
        return Duration::from_secs(self.warmup);
    }
}

async fn do_run(
    mut client: EchoerClient<Channel>,
    request: tonic::Request<EchoRequest>,
) -> anyhow::Result<Duration> {
    let start = tokio::time::Instant::now();
    let _reply = client.echo(request).await?;
    Ok(start.elapsed())
}

async fn closed_client(id: &str, args: &Args) -> anyhow::Result<()> {
    let client = EchoerClient::connect(format!("http://{}:{}", args.host, args.port)).await?;
    tracing::info!("connected @ {}:{}", args.host, args.port);
    let request = EchoRequest {
        msg: vec![42u8; args.message_size].into(),
    };

    let mut reporting = false;
    let start = std::time::Instant::now();
    while start.elapsed() < args.duration() + args.warmup() {
        let elapsed = do_run(client.clone(), tonic::Request::new(request.clone())).await?;
        if !reporting && start.elapsed() > args.warmup() {
            reporting = true;
            println!("Start: {} {:.9}", id, start.elapsed().as_secs_f64());
        }

        if reporting && start.elapsed() < args.duration() + args.warmup() {
            println!("{:.3}", elapsed.as_secs_f64() * 1_000_000f64);
        }
    }

    println!("End: {} {:.9}", id, start.elapsed().as_secs_f64());

    Ok(())
}

async fn run_closed(id: String, args: Args) -> anyhow::Result<()> {
    let paralellism = args.n_cores.unwrap_or_else(|| num_cpus::get());
    let runners = (0..paralellism)
        .into_iter()
        .map(|_| closed_client(&id, &args))
        .collect::<Vec<_>>();
    futures::future::join_all(runners)
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(())
}

async fn run_bursty(id: String, args: Args) -> anyhow::Result<()> {
    let paralellism = args.n_cores.unwrap_or_else(|| num_cpus::get());
    let client = EchoerClient::connect(format!("http://{}:{}", args.host, args.port)).await?;
    tracing::info!("connected @ {}:{}", args.host, args.port);
    let request = EchoRequest {
        msg: vec![42u8; args.message_size].into(),
    };

    let mut reporting = false;
    let start = tokio::time::Instant::now();
    while start.elapsed() < args.duration() + args.warmup() {
        if !reporting && start.elapsed() > args.warmup() {
            reporting = true;
            println!("Start: {} {:.9}", id, start.elapsed().as_secs_f64());
        }
        let futs = (0..paralellism)
            .into_iter()
            .map(|_| do_run(client.clone(), tonic::Request::new(request.clone())))
            .collect::<Vec<_>>();

        futures::future::join_all(futs)
            .await
            .into_iter()
            .map(|x| {
                x.map(|elapsed| {
                    if reporting && start.elapsed() < args.duration() + args.warmup() {
                        println!("{:.3}", elapsed.as_secs_f64() * 1_000_000f64);
                    }
                })
            })
            .collect::<anyhow::Result<Vec<()>>>()?;
    }
    println!("End: {} {:.9}", id, start.elapsed().as_secs_f64());

    Ok(())
}

async fn run(id: String, args: Args) -> anyhow::Result<()> {
    match args.client_type {
        ClientType::Bursty => run_bursty(id, args).await,
        ClientType::Closed => run_closed(id, args).await,
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .init();
    let args = Args::parse();
    let id = format!(
        "{}:{:x}",
        gethostname::gethostname()
            .into_string()
            .map_err(|os_str| anyhow::anyhow!("failed to convert hostname: {:?}", os_str))?,
        uuid::Uuid::new_v4().simple()
    );

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

    println!("Message Size: {}", args.message_size);
    if let Some(start) = args.start {
        let now_ts = Local::now().timestamp_nanos_opt().ok_or(anyhow::anyhow!("an i64 can represent stuff until 2262. if you are still using this code in 2262 first of all, thanks i guess; second, i don't really care, probably fix this"))?;
        let start_ts = start.timestamp_nanos_opt().ok_or(anyhow::anyhow!("an i64 can represent stuff until 2262. if you are still using this code in 2262 first of all, thanks i guess; second, i don't really care, probably fix this"))?;
        if now_ts < start_ts {
            std::thread::sleep(std::time::Duration::from_nanos((start_ts - now_ts) as u64));
        }
    }
    rt.block_on(run(id, args))
}
