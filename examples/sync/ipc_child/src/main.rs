use std::env;
use std::thread;

use clap::Parser;
use ipc_channel_adapter::child::sync::HostReceiver;
use ipc_channel_adapter::child::sync::HostSender;

#[derive(Parser, Debug, Clone)]
struct Config {
  #[arg(short = 'b', long = "benchmark", env = "IPC_BENCHMARK")]
  pub benchmark: bool,
}

fn main() {
  let config = Config::parse();

  let host_receiver = HostReceiver::<usize, usize>::new(&env::var("IPC_CHANNEL_HOST_OUT").unwrap());
  let host_sender =
    HostSender::<usize, usize>::new(&env::var("IPC_CHANNEL_HOST_IN").unwrap()).unwrap();

  // If not running benchmark
  if !config.benchmark {
    let rx = host_receiver.on.subscribe();

    thread::spawn(move || {
      while let Ok((v, reply)) = rx.recv() {
        println!("[Child] Received: {}", v);
        reply.send(v).unwrap()
      }
    });

    let response = host_sender.send_blocking(43);
    println!("[Child] Response: {}", response);
    return;
  }

  // Benchmark responder
  let rx = host_receiver.on.subscribe();
  while let Ok((v, reply)) = rx.recv() {
    reply.send(v).unwrap()
  }
}
