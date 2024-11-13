use std::env;

use clap::Parser;
use ipc_channel_adapter::ipc::sync::ClientConnection;



#[derive(Parser, Debug, Clone)]
struct Config {
  #[arg(short = 'b', long = "benchmark", env = "IPC_BENCHMARK")]
  pub benchmark: bool,
}

fn main() {
  let config = Config::parse();

  let host_sender_server = env::var("IPC_CHANNEL_HOST_SERVER").unwrap();

  let rx = ClientConnection::<usize, usize>::new(&host_sender_server).unwrap();

    // Benchmark responder
  while let Ok((v, reply)) = rx.recv() {
    reply.send(v).unwrap()
  }
}