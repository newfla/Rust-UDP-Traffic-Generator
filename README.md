# Rust UDP Traffic Generator

A CLI tool to generate UDP traffic based on [Tokio framework](https://https://tokio.rs).

# Cargo Install

```
cargo install udp_traffic_generator
```

# Help

```
./udp_traffic_generator --help
Simple stress test for UDP Server

Usage: udp_traffic_generator [OPTIONS] --destination <addr>

Options:
  -d, --destination <addr>     Server address as IP:PORT
  -c, --connections <clients>  Number of clients to simulate [default: 1]
  -l, --length <length>        Payload size as bytes [default: 16]
  -r, --rate <rate>            Defined as packets/sec [default: 1]
  -p, --port <port>            Starting source port for clients [default: 8000]
  -w, --workers <workers>      Number of worker threads for the Tokio runtime [default: #CPU core]
  -s, --sleep time <timeout>   Timeout between consecutive connections spawn as ms [default: 50]
  -h, --help                   Print help information
  -V, --version                Print version information