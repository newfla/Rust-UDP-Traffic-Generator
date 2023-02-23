use std::{net::SocketAddr, time::Duration,iter::repeat_with, io::Error};

use log::error;
use openssl::ssl::{SslMethod, SslContext};
use statistics::stats_task;
use sender::{sender_task_plain, sender_task_dtls};
use tokio::{net::UdpSocket, time::sleep, task::JoinSet};
use derive_new::new;
use tokio_dtls_stream_sink::{Session, Client};

mod statistics;
mod sender;


pub async fn manager (params: Parameters) {
    let (use_dtls, ca_file) = params.dtls;
    if use_dtls && ca_file.is_none() {
        error!("DTLS requires CA file to verify server credentials");
        return ;
    }

    let stats_tx = stats_task(params.connections);

    let mut tasks = JoinSet::new();
    let mut start_port = params.start_port; 

    for id in 0..params.connections {
        let payload = generate_payloads(params.len);
        let stats_tx_cloned = stats_tx.clone();
        let ca_file= ca_file.clone();
        if use_dtls {
            let session = setup_dtls_session(start_port,params.server_addr,ca_file.unwrap()).await;
            tasks.spawn(async move {
                sender_task_dtls(id, session, payload, params.rate, stats_tx_cloned).await
            });
        } else {
            let socket = setup_socket(params.server_addr, start_port).await;
            tasks.spawn(async move {
                sender_task_plain(id, socket, payload, params.rate, stats_tx_cloned).await
            });
        }
        
        start_port+=1;
        sleep(Duration::from_millis(params.sleep)).await;
    }
    while (tasks.join_next().await).is_some() {

    }
}

async fn setup_socket(addr: SocketAddr,port: usize) -> UdpSocket{
    let socket = UdpSocket::bind("0.0.0.0:".to_owned() + &port.to_string()).await.unwrap();
    socket.connect(addr).await.unwrap();
    socket
}

async fn setup_dtls_session(port: usize, addr: SocketAddr, ca_file: String) -> DtlsSession {
    let mut ctx = SslContext::builder(SslMethod::dtls()).unwrap();
    ctx.set_ca_file(ca_file).unwrap();
    let socket = UdpSocket::bind("0.0.0.0:".to_owned() + &port.to_string()).await.unwrap();
    let client = Client::new(socket);
    let session = client.connect(addr, Some(ctx.build())).await.unwrap();
    DtlsSession::new(client,session)
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
    start_port: usize,
    sleep: u64,
    dtls: (bool, Option<String>)
}

#[derive(new)]
pub struct DtlsSession {
    _client: Client,
    session: Session
}

impl DtlsSession {
    pub async fn write(&mut self, buf: &[u8]) ->Result<(), Error> {
        self.session.write(buf).await
    }
}
