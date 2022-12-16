use std::time::{Duration, Instant};

use kanal::AsyncSender;
use log::debug;
use tokio::{net::UdpSocket, time::sleep};

use crate::statistics::StatPacket;

pub async fn sender_task(id: usize, socket: UdpSocket, payload: &'static [u8], rate: usize, stats_tx: AsyncSender<StatPacket>){
    debug!("client {} spawned",id);
    let one_sec = Duration::new(1,0);
    loop { 
        let start_time = Instant::now();
        let mut packets_error = 0;

        for _ in 0..rate {
            if socket.send(payload).await.is_err() {
                packets_error+=1;
            }
        }
        let packets_sent = rate - packets_error;
        let _ = stats_tx.send((packets_sent * payload.len(),packets_sent)).await;

        let time_elapsed = Instant::now() - start_time;

        if time_elapsed < one_sec {
            let time_to_sleep = one_sec - time_elapsed;
            sleep(time_to_sleep).await;
        }
    }
}
