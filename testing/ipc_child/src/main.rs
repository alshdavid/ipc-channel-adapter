use std::env;
use std::thread;

use ipc_adapter::child::sync::HostReceiver;
use ipc_adapter::child::sync::HostSender;

fn main() {
  let host_receiver = HostReceiver::<usize, usize>::new(&env::var("IPC_CHANNEL_HOST_OUT").unwrap());
  let host_sender =
    HostSender::<usize, usize>::new(&env::var("IPC_CHANNEL_HOST_IN").unwrap()).unwrap();

  let rx = host_receiver.on.subscribe();

  thread::spawn(move || {
    while let Ok((v, reply)) = rx.recv() {
      println!("[Child] Received: {}", v);
      reply.send(v).unwrap()
    }
  });

  let response = host_sender.send_blocking(43);
  println!("[Child] Response: {}", response);
}
