# Rust UDP Traffic Generator

A CLI tool to generate UDP and DTLS traffic based on [Tokio framework](https://https://tokio.rs).

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
  -s, --timeout <timeout>      Timeout between consecutive connections spawn as ms [default: 50]
      --dtls <dtls>            Send data over DTLS [default: false] [possible values: true, false]
      --ca <ca>                PEM File to validate server credentials
  -h, --help                   Print help
  -V, --version                Print version