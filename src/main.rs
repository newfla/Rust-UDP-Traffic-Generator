use std::net::ToSocketAddrs;

use byte_unit::Byte;
use log::{info,warn, LevelFilter};
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
        .about("Simple stress test for UDP Server")
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
        )
        .get_matches()
}

fn build_runtime(cli: &ArgMatches) -> Runtime {
    let mut rt = Runtime::new().unwrap();
    let workers = cli.get_one::<usize>("workers");

    if let Some(workers) = workers {
        if *workers > 0 {
            rt = Builder::new_multi_thread()
                .worker_threads(*workers)
                .enable_all()
                .build()
                .unwrap();
        } else {
            warn!("Workers threads must be > 0. Switching to #CPU Core");
        }
    }
    rt
}

fn extract_parameters(matches: ArgMatches) -> Parameters {
    let server_addr =  matches.get_one::<String>("addr").unwrap().to_socket_addrs().unwrap().next().unwrap();
    let rate: usize = *matches.get_one("rate").unwrap();
    let connections :usize = *matches.get_one("clients").unwrap();
    let len: usize= *matches.get_one("length").unwrap();
    let start_port: usize = *matches.get_one("port").unwrap();

    let bandwidth = Byte::from_bytes((connections * rate * len * 8) as u128).get_appropriate_unit(false).to_string();
    let bandwidth = bandwidth[0..bandwidth.len()-1].to_string();

    info!("Server address: {}, clients: {}, payload size: {}, rate: {}",server_addr, connections, len, rate);
    info!("Theoretical Packets rate: {} pks/sec, Theoretical Bandwidth: {}bit/s", connections * rate, bandwidth);

    Parameters::new( server_addr, rate, connections, len, start_port)
}