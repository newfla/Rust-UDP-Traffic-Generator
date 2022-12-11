use std::{net::SocketAddr, time::Duration,iter::repeat_with};

use statistics::stats_task;
use sender::sender_task;
use tokio::{net::UdpSocket, time::sleep, task::JoinSet};
use derive_new::new;

mod statistics;
mod sender;

pub async fn manager (params: Parameters) {
    

    let stats_tx = stats_task(params.connections);

    let mut tasks = JoinSet::new();
    let mut start_port = params.start_port; 

    for id in 0..params.connections {
        let socket = setup_socket(start_port,params.server_addr).await;
        let payload = generate_payloads(params.len);
        let stats_tx_cloned = stats_tx.clone();
        let x= Box::leak(Box::new(payload));
        tasks.spawn(async move {
            sender_task(id, socket, x, params.rate, stats_tx_cloned).await
        });
        start_port+=1;
        sleep(Duration::from_millis(50)).await;
    }
    while (tasks.join_next().await).is_some() {

    }
}

async fn setup_socket(port: usize, addr: SocketAddr) -> UdpSocket{
    let socket = UdpSocket::bind("0.0.0.0:".to_owned()+ &port.to_string()).await.unwrap();
    socket.connect(addr).await.unwrap();
    socket
}

fn generate_payloads(len: usize) -> Vec<u8>{
    repeat_with(|| fastrand::u8(..)).take(len).collect()
}

#[derive(new)]
pub struct Parameters {
    server_addr: SocketAddr,
    rate: usize,
    connections: usize,
    len: usize,
    start_port: usize
}

