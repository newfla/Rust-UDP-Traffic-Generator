use std::net::SocketAddr;

use rand::{thread_rng, distributions::Alphanumeric, Rng};
use statistics::stats_task;
use sender::sender_task;
use tokio::{net::UdpSocket, spawn, task::JoinSet};

mod statistics;
mod sender;

pub async fn manager (params: Parameters) {
    

    let stats_tx = stats_task();

    let mut tasks = JoinSet::new();
    let mut start_port = params.start_port; 

    for id in 0..params.connections {
        let socket = setup_socket(start_port).await;
        let payload = generate_payloads(params.len);
        let stats_tx_cloned = stats_tx.clone();
        tasks.spawn(async move {
            sender_task(id, socket, params.server_addr, payload, params.rate, stats_tx_cloned).await
        });
        start_port+=1;
    }
    while (tasks.join_next().await).is_some() {

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


pub struct Parameters {
    server_addr: SocketAddr,
    rate: usize,
    connections: usize,
    len: usize,
    start_port: usize
}

impl Parameters {
    pub fn new(server_addr: SocketAddr,
        rate: usize,
        connections: usize,
        len: usize,
        start_port: usize) -> Self {
            Self { server_addr, rate, connections, len, start_port }
        }
}

