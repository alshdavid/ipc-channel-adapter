use std::env;

use clap::Parser;
use ipc_channel_adapter::child::asynch::HostReceiver;
use ipc_channel_adapter::child::asynch::HostSender;

#[derive(Parser, Debug, Clone)]
struct Config {
  #[arg(short = 'b', long = "benchmark", env = "IPC_BENCHMARK")]
  pub benchmark: bool,
}

#[tokio::main]
async fn main() {
  let config = Config::parse();

  // Send requests to host
  let host_sender_server = env::var("IPC_CHANNEL_HOST_IN").unwrap();
  let host_sender = HostSender::<usize, usize>::new(&host_sender_server).unwrap();

  // Receive requests from host
  let host_receiver_server = env::var("IPC_CHANNEL_HOST_OUT").unwrap();
  let (_host_receiver, mut host_receiver_rx) =
    HostReceiver::<usize, usize>::new(&host_receiver_server).unwrap();

  // If not running benchmark
  if !config.benchmark {
    tokio::spawn(async move {
      while let Some((v, reply)) = host_receiver_rx.recv().await {
        println!("[Child] Received: {}", v);
        reply.send(v).unwrap()
      }
    });

    let response = host_sender.send_and_wait(43).await;
    println!("[Child] Response: {}", response);

    return;
  }

  // Benchmark responder
  while let Some((v, reply)) = host_receiver_rx.recv().await {
    reply.send(v).unwrap()
  }
  println!("done");
}
