use std::net::ToSocketAddrs;

use byte_unit::Byte;
use log::{info, warn, LevelFilter};
use simple_logger::SimpleLogger;
use clap::{Arg, ArgMatches, Command};
use tokio::runtime::{Builder, Runtime};
use udp_traffic_generator::{manager, Parameters};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {

    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    let cli = build_cli();
    let rt = build_runtime(&cli);
   
    rt.block_on(async {manager(extract_parameters(cli)).await;});
}
fn build_cli() -> ArgMatches {
    Command::new("UDP TRAFFIC GENERATOR")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("Simple stress test for UDP/DTLS Server")
        .arg(
            Arg::new("addr")
                .short('d')
                .long("destination")
                .help("Server address as IP:PORT")
                .required(true)
        )
        .arg(
            Arg::new("clients")
                .short('c')
                .long("connections")
                .help("Number of clients to simulate")
                .default_value("1")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::new("length")
                .short('l')
                .long("length")
                .help("Payload size as bytes")
                .default_value("16")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::new("rate")
                .short('r')
                .long("rate")
                .help("Defined as packets/sec")
                .default_value("1")
                .value_parser(clap::value_parser!(usize))

        ).arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .help("Starting source port for clients")
                .default_value("8000")
                .value_parser(clap::value_parser!(usize))
        ).arg(
            Arg::new("workers")
                .short('w')
                .long("workers")
                .help("Number of worker threads for the Tokio runtime [default: #CPU core]")
                .value_parser(clap::value_parser!(usize))
        ).arg(
            Arg::new("timeout")
                .short('s')
                .long("timeout")
                .help("Timeout between consecutive connections spawn as ms")
                .default_value("50")
                .value_parser(clap::value_parser!(u64))
        ).arg(
            Arg::new("dtls")
                .long("dtls")
                .help("Send data over DTLS")
                .default_value("false")
                .value_parser(clap::value_parser!(bool))
        ).arg(
            Arg::new("ca")
                .long("ca")
                .help("PEM File to validate server credentials")
                .value_parser(clap::value_parser!(String))
        )
        .get_matches()
}

fn build_runtime(cli: &ArgMatches) -> Runtime {

    let worker_threads = cli.get_one::<usize>("workers");
    let mut rt_builder = Builder::new_multi_thread();
    if let Some(workers) = worker_threads {
        if *workers > 0 {
            rt_builder.worker_threads(*workers);
        }

    } else {
        warn!("Workers threads must be > 0. Switching to #CPU Core");
    }

    rt_builder.enable_all()
              .build()
              .unwrap()
}

fn extract_parameters(matches: ArgMatches) -> Parameters {
    let server_addr =  matches.get_one::<String>("addr").unwrap().to_socket_addrs().unwrap().next().unwrap();
    let rate: usize = *matches.get_one("rate").unwrap();
    let connections: usize = *matches.get_one("clients").unwrap();
    let len: usize = *matches.get_one("length").unwrap();
    let start_port: usize = *matches.get_one("port").unwrap();
    let sleep: u64 = *matches.get_one("timeout").unwrap();

    let bandwidth = Byte::from_bytes((connections * rate * len * 8) as u128).get_appropriate_unit(false).to_string();
    let bandwidth = bandwidth[0..bandwidth.len()-1].to_string();

    let use_dtls: bool = *matches.get_one("dtls").unwrap();
    let ca_file = matches.get_one("ca").cloned();

    info!("Server address: {}, clients: {}, payload size: {}, rate: {} pkt/s, sleep timeout:{} ms, dtls: {}",server_addr, connections, len, rate, sleep, use_dtls);
    info!("Theoretical Packets rate: {} pkt/sec", connections * rate);
    info!("Theoretical Bandwidth: {}bit/s", bandwidth);

    Parameters::new(server_addr, rate, connections, len, start_port, sleep, (use_dtls, ca_file))
}