# IPC Channel Adapter

This utility wraps [ipc-channel](https://github.com/servo/ipc-channel) to handle the handshake and respond with standard channels.

It supports Tokio for non-blocking operation

## Usage

#### Parent Process

```rust
fn main() {
  // Send requests to child
  //   Request to send to child is u32 
  //   Response coming back from child u64 
  let child_sender = ChildSender::<u32, u64>::new().unwrap();

  // Receive requests from child
  //   Request coming from child is u32 
  //   Response to send back to child is u64 
  let (child_receiver, child_rx) = ChildReceiver::<u32, u64>::new().unwrap();

  let child_process = spawn_child_process("your_child_process")
    .env("IPC_RECEIVER_NAME", child_receiver.server_name)
    .env("IPC_SENDER_NAME", child_sender.server_name)
    .spawn();

  thread::spawn(move || {
    while let Ok((v, reply)) = child_rx.recv() {
      println!("[Host] Received: {}", v);
      reply.send(v).unwrap()
    }
  });

  let response = child_sender.send_blocking(42);
  println!("[Host] Response: {}", response);
}
```

#### Child Process

```rust
fn main() {
  // Get name of IPC servers
  let host_receiver_server = env::var("IPC_RECEIVER_NAME").unwrap();
  let host_sender_server = env::var("IPC_SENDER_NAME").unwrap();
  
  // Send requests to host
  //   Request to send to host is u32 
  //   Response coming back from host u64 
  let host_sender = HostSender::<u32, u64>::new(&host_receiver_server).unwrap();

  // Receive requests from host
  //   Request coming from host is u32 
  //   Response to send back to host is u64 
  let (_host_receiver, host_receiver_rx) = HostReceiver::<u32, u64>::new(&host_sender_server).unwrap();

  thread::spawn(move || {
    while let Ok((v, reply)) = host_receiver_rx.recv() {
      println!("[Child] Received: {}", v);
      reply.send(v).unwrap()
    }
  });

  let response = host_sender.send_blocking(43);
  println!("[Child] Response: {}", response);
}
```
