use std::time::Duration;

use byte_unit::Byte;
use kanal::{AsyncSender, bounded_async};
use log::info;
use tokio::{spawn, time::interval, select};

pub type StatPacket = (usize, usize);

pub fn stats_task(clients: usize) -> AsyncSender<StatPacket> {
    //Define channel to send statistics update
    let (stats_tx,stats_rx) = bounded_async(clients);

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
                    if let Ok((bytes,packets)) = stat {
                        bytes_sent += bytes as f64;
                        packets_sent += packets;
                    }
                }
            }
        }
    });
    stats_tx
}
