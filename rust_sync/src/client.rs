use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;

use anyhow::Context;
use chrono::{Local, NaiveDateTime, NaiveTime};
use clap::Parser;

const BUFFER_SIZE: usize = 1 << 16;

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

#[derive(Parser, Clone)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(default_value = "[::1]")]
    host: String,

    #[arg(default_value_t = 9094, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,

    #[arg(short = 'j', long)]
    n_cores: Option<usize>,

    #[arg(short, long, default_value_t = 1000)]
    reps: usize,

    #[arg(short = 's', long, default_value_t = 1, value_parser = size_parser)]
    message_size: usize,

    #[arg(long, value_parser = time_parser)]
    start: Option<NaiveDateTime>,
}

fn do_run(stream: &mut TcpStream, message_size: usize) -> anyhow::Result<()> {
    let mut buffer = [42; BUFFER_SIZE];
    let start = std::time::Instant::now();
    let mut need_to_write = message_size;
    while need_to_write > 0 {
        let n = stream.write(&buffer[..std::cmp::min(need_to_write, BUFFER_SIZE)])?;
        need_to_write -= n;
    }
    let mut waiting_for = message_size;
    while waiting_for > 0 {
        buffer.fill(0);
        let n = stream.read(&mut buffer[..std::cmp::min(waiting_for, BUFFER_SIZE)])?;
        if !buffer[..n].iter().all(|x| *x == 42) {
            return Err(anyhow::anyhow!("mismatched reply"));
        }
        waiting_for -= n;
    }
    println!("{:.3}", start.elapsed().as_secs_f64() * 1_000_000f64);
    Ok(())
}

fn closed_client(args: Args, reps: usize) -> anyhow::Result<()> {
    match TcpStream::connect(format!("{}:{}", args.host, args.port)) {
        Ok(mut stream) => {
            let size = args.message_size.to_be_bytes();
            stream.write_all(&size)?;
            for _ in 0..reps {
                do_run(&mut stream, args.message_size)?
            }
        }
        Err(e) => {
            return Err(e).context("failed to connect");
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .finish();
    let args = Args::parse();

    let paralellism = args.n_cores.unwrap_or_else(|| num_cpus::get());

    if let Some(start) = args.start {
        let now_ts = Local::now().timestamp_nanos_opt().ok_or(anyhow::anyhow!("an i64 can represent stuff until 2262. if you are still using this code in 2262 first of all, thanks i guess; second, i don't really care, probably fix this"))?;
        let start_ts = start.timestamp_nanos_opt().ok_or(anyhow::anyhow!("an i64 can represent stuff until 2262. if you are still using this code in 2262 first of all, thanks i guess; second, i don't really care, probably fix this"))?;
        if now_ts < start_ts {
            std::thread::sleep(std::time::Duration::from_nanos((start_ts - now_ts) as u64));
        }
    }

    let start = std::time::Instant::now();
    let runners = (0..paralellism)
        .into_iter()
        .map(|idx| {
            if idx == 0 {
                args.reps / paralellism + args.reps % paralellism
            } else {
                args.reps / paralellism
            }
        })
        .map(|reps| (args.clone(), reps))
        .map(|(args, reps)| thread::spawn(move || closed_client(args, reps)))
        .collect::<Vec<_>>();

    runners
        .into_iter()
        .filter_map(|x| match x.join() {
            Ok(ok) => Some(ok),
            Err(e) => {
                tracing::warn!("join error: {:?}", e);
                None
            }
        })
        .collect::<anyhow::Result<_>>()?;
    let elapsed = start.elapsed();

    println!("Elapsed: {:.9}", elapsed.as_secs_f64());
    println!("Message Size: {}", args.message_size);

    Ok(())
}
