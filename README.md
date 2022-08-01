# Rust UDP Traffic Generator

A CLI tool to generate UDP traffic based on [Tokio framework](https://https://tokio.rs).


```
./udp_traffic_generator --help
UDP TRAFFIC GENERATOR 0.1.0
Simple stress test for UDP Server

USAGE:
    udp_traffic_generator [OPTIONS] --destination <addr>

OPTIONS:
    -c, --connections <clients>    Number of clients to simulate [default: 1]
    -d, --destination <addr>       Server address as IP:PORT
    -h, --help                     Print help information
    -l, --length <length>          Payload size as bytes [default: 16]
    -p, --port <port>              Starting source port for clients [default: 8000]
    -r, --rate <rate>              Defined as packets/sec [default: 1]
    -V, --version                  Print version information
    -w, --workers <workers>        Number of worker threads for the Tokio runtime [default: #CPU core]
```