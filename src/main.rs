use std::net::{SocketAddr, ToSocketAddrs};
use std::time::{Instant, Duration};

use log::{info,warn};
use simple_logger::SimpleLogger;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use clap::{App, Arg, ArgMatches};
use tokio::runtime::{Builder, Runtime};
use tokio::{net::UdpSocket,time::{sleep,interval},task::{JoinSet,spawn},sync::mpsc::{channel,Sender},select};
use byte_unit::Byte;

type StatPacket = (usize, usize);

fn main() {
    SimpleLogger::new().init().unwrap();

    let cli = build_cli();
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
    
    rt.block_on(async {start_up_client(cli).await;});
}

fn start_stats_task() -> Sender<StatPacket> {
    //Define channel to send statistics update
    let (stats_tx,mut stats_rx) = channel(100);

    spawn(async move {
        let timer_duration = 10.;
        let mut timer = interval(Duration::from_secs(timer_duration as u64));

        let mut bytes_sent = 0.;
        let mut packets_sent = 0;
        loop {
            select! {
                _ = timer.tick() => {
                    bytes_sent*=8.;
                    let bandwidth = Byte::from_bytes((bytes_sent / timer_duration) as u128).get_appropriate_unit(false).to_string();
                    let bandwidth = bandwidth[0..bandwidth.len()-1].to_string();
                    info!("Sent {} packets --- Bandwidth {}bit/s", packets_sent, bandwidth);
                    bytes_sent = 0.;
                    packets_sent = 0;
                }
                stat = stats_rx.recv() => {
                    if let Some((bytes,packets)) = stat {
                        bytes_sent += bytes as f64;
                        packets_sent += packets;
                    }
                }
            }
        }
    });

    stats_tx

}

async fn start_up_client(matches: ArgMatches) {
    let server_addr =  matches.get_one::<String>("addr").unwrap().to_socket_addrs().unwrap().next().unwrap();
    let rate: usize = *matches.get_one("rate").unwrap();
    let connections :usize = *matches.get_one("clients").unwrap();
    let len: usize= *matches.get_one("length").unwrap();
    let mut startport: usize = *matches.get_one("port").unwrap();

    let stats_tx = start_stats_task();
    let mut tasks = JoinSet::new();
    
    info!("Server addr: {}, clients: {}, payload size: {}, rate: {}",server_addr, connections, len, rate);

    for id in 0..connections {
        let socket = setup_socket(startport).await;
        let payload = generate_payloads(len);
        let stats_tx_cloned = stats_tx.clone();
        tasks.spawn(async move {
            client_sender_function(id, socket, server_addr, payload, rate, stats_tx_cloned).await
        });
        startport+=1;
    }

    while (tasks.join_next().await).is_some() {
        
    }
}

async fn client_sender_function(id: usize, socket: UdpSocket, server: SocketAddr, payload: Vec<u8>, rate: usize, stats_tx: Sender<StatPacket>){
    info!("client {} spawned",id);
    let one_sec = Duration::new(1,0);
    loop { 
        let start_time = Instant::now();
        let mut bytes_sent = 0;
        let mut packets_sent = 0;

        for _ in 0..rate {
            let bytes = socket.send_to(&payload, server).await;
            if let Ok(bytes) = bytes {
                bytes_sent += bytes;
                packets_sent+=1;
            }
        }

        let _ = stats_tx.send((bytes_sent,packets_sent)).await;

        let time_elapsed = Instant::now() - start_time;

        if time_elapsed < one_sec {
            let time_to_sleep = one_sec - time_elapsed;
            sleep(time_to_sleep).await;
        }
    }
}

async fn setup_socket(port: usize) -> UdpSocket{
    UdpSocket::bind("0.0.0.0:".to_owned()+ &port.to_string()).await.unwrap()
}

fn generate_payloads(len: usize) -> Vec<u8>{
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .collect()
}

fn build_cli() -> ArgMatches {
    App::new("UDP TRAFFIC GENERATOR")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("Simple stress test for UDP Server")
        .arg(
            Arg::with_name("addr")
                .short('d')
                .long("destination")
                .help("Server address as IP:PORT")
                .takes_value(true)
                .required(true)
        )
        .arg(
            Arg::with_name("clients")
                .short('c')
                .long("connections")
                .help("Number of clients to simulate")
                .takes_value(true)
                .default_value("1")
                .value_parser(clap::value_parser!(usize))
        )

        .arg(
            Arg::with_name("length")
                .short('l')
                .long("length")
                .help("Payload size as bytes")
                .takes_value(true)
                .default_value("16")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::with_name("rate")
                .short('r')
                .long("rate")
                .help("Defined as packets/sec")
                .takes_value(true)
                .default_value("1")
                .value_parser(clap::value_parser!(usize))

        ).arg(
            Arg::with_name("port")
                .short('p')
                .long("port")
                .help("Starting source port for clients")
                .takes_value(true)
                .default_value("8000")
                .value_parser(clap::value_parser!(usize))
        ).arg(
            Arg::with_name("workers")
                .short('w')
                .long("workers")
                .help("Number of worker threads for the Tokio runtime [default: #CPU core]")
                .takes_value(true)
                .value_parser(clap::value_parser!(usize))
        )
        .get_matches()
}
