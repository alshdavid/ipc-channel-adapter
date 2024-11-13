// use std::env;
// use std::thread;

// use clap::Parser;
// use ipc_channel_adapter::child::sync::HostReceiver;
// use ipc_channel_adapter::child::sync::HostSender;

// #[derive(Parser, Debug, Clone)]
// struct Config {
//   #[arg(short = 'b', long = "benchmark", env = "IPC_BENCHMARK")]
//   pub benchmark: bool,
// }

// fn main() {
//   let config = Config::parse();

//   // Send requests to host
//   let host_sender_server = env::var("IPC_CHANNEL_HOST_IN").unwrap();
//   let host_sender = HostSender::<usize, usize>::new(&host_sender_server).unwrap();

//   // Receive requests from host
//   let host_receiver_server = env::var("IPC_CHANNEL_HOST_OUT").unwrap();
//   let (_host_receiver, host_receiver_rx) =
//     HostReceiver::<usize, usize>::new(&host_receiver_server).unwrap();

//   // If not running benchmark
//   if !config.benchmark {
//     thread::spawn(move || {
//       while let Ok((v, reply)) = host_receiver_rx.recv() {
//         println!("[Child] Received: {}", v);
//         reply.send(v).unwrap()
//       }
//     });

//     let response = host_sender.send_blocking(43);
//     println!("[Child] Response: {}", response);

//     return;
//   }

//   // Benchmark responder
//   while let Ok((v, reply)) = host_receiver_rx.recv() {
//     reply.send(v).unwrap()
//   }
//   println!("done");
// }

fn main() {}