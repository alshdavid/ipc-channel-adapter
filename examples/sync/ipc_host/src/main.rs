use std::env;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::SystemTime;

use clap::Parser;
use ipc_channel_adapter::host::sync::ChildReceiver;
use ipc_channel_adapter::host::sync::ChildSender;

#[derive(Parser, Debug, Clone)]
struct Config {
  #[arg(short = 'b', long = "benchmark", env = "IPC_BENCHMARK")]
  pub benchmark: bool,

  #[arg(
    short = 'm',
    long = "benchmark-message-count",
    env = "IPC_BENCHMARK_MESSAGE_COUNT",
    default_value = "100000"
  )]
  pub benchmark_message_count: usize,
}

fn main() {
  let config = Config::parse();

  // Send requests to child
  let child_sender = ChildSender::<usize, usize>::new().unwrap();

  // Receive requests from child
  let (child_receiver, child_rx) = ChildReceiver::<usize, usize>::new().unwrap();

  let mut entry = std::env::current_exe()
    .unwrap()
    .parent()
    .unwrap()
    .to_owned();

  if env::consts::OS == "windows" {
    entry = entry.join("ipc_child_sync.exe");
  } else {
    entry = entry.join("ipc_child_sync");
  }

  let mut command = Command::new(entry.to_str().unwrap());
  command.env("IPC_CHANNEL_HOST_OUT", &child_sender.server_name);
  command.env("IPC_CHANNEL_HOST_IN", &child_receiver.server_name);
  command.env("IPC_BENCHMARK", &config.benchmark.to_string());

  command.stderr(Stdio::inherit());
  command.stdout(Stdio::inherit());
  command.stdin(Stdio::piped());

  command.spawn().unwrap();

  // If not running benchmark
  if !config.benchmark {
    thread::spawn(move || {
      while let Ok((v, reply)) = child_rx.recv() {
        println!("[Host] Received: {}", v);
        reply.send(v).unwrap()
      }
    });

    let response = child_sender.send_blocking(42).unwrap();
    println!("[Host] Response: {}", response);
    return;
  }

  println!(
    "Benchmark: host sending \"{}\" messages",
    config.benchmark_message_count
  );

  // Benchmark mode
  let expect = 1 * config.benchmark_message_count;
  let mut sum = 0;

  let start_time = SystemTime::now();
  for _ in 0..config.benchmark_message_count {
    let result = child_sender.send_blocking(1).unwrap();
    sum += result;
  }
  let end_time = start_time.elapsed().unwrap();

  assert!(sum == expect, "Expected sums to match");

  println!(
    "Total Time (ms): {:.3}s",
    end_time.as_nanos() as f64 / 1_000_000 as f64 / 1000 as f64
  );
}
