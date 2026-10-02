use std::env;
use std::net::UdpSocket;
use std::process;
use std::sync::atomic::AtomicBool;

use echlub_jam_relay::{run, RelayConfig};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut bind = "0.0.0.0:7400".to_string();
    let mut config = RelayConfig::default();
    let mut i = 1;
    while i < args.len() {
        let value = args.get(i + 1).cloned();
        match (args[i].as_str(), value) {
            ("--bind", Some(v)) => bind = v,
            ("--depth", Some(v)) => config.jitter_depth = v.parse().unwrap_or_else(|_| usage()),
            ("--max-peers", Some(v)) => config.max_peers = v.parse().unwrap_or_else(|_| usage()),
            ("--mode", Some(v)) => {
                config.forward = match v.as_str() {
                    "mix" => false,
                    "forward" => true,
                    _ => usage(),
                }
            }
            _ => usage(),
        }
        i += 2;
    }
    let socket = UdpSocket::bind(&bind).unwrap_or_else(|e| {
        eprintln!("bind {bind}: {e}");
        process::exit(1);
    });
    eprintln!(
        "jam-relay listening on {} (mode {}, depth {}, max peers {}) - prototype, no auth",
        socket.local_addr().expect("bound"),
        if config.forward { "forward" } else { "mix" },
        config.jitter_depth,
        config.max_peers
    );
    let stop = AtomicBool::new(false);
    match run(socket, config, &stop) {
        Ok(stats) => eprintln!("{stats:?}"),
        Err(e) => {
            eprintln!("relay error: {e}");
            process::exit(1);
        }
    }
}

fn usage() -> ! {
    eprintln!(
        "Usage: jam-relay [--bind 0.0.0.0:7400] [--mode mix|forward] [--depth 2] [--max-peers 4]"
    );
    process::exit(2);
}
