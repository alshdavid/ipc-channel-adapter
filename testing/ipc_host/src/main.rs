use ipc_adapter::host::sync::ChildReceiver;
use ipc_adapter::host::sync::ChildSender;
use std::process::Command;
use std::process::Stdio;
use std::thread;

fn main() {
    let child_sender = ChildSender::<usize, usize>::new();
    let child_receiver = ChildReceiver::<usize, usize>::new();

    let entry = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("ipc_child")
        .to_owned();

    let mut command = Command::new(entry.to_str().unwrap());
    command.env("IPC_CHANNEL_HOST_OUT", &child_sender.server_name);
    command.env("IPC_CHANNEL_HOST_IN", &child_receiver.server_name);

    command.stderr(Stdio::inherit());
    command.stdout(Stdio::inherit());
    command.stdin(Stdio::piped());

    command.spawn().unwrap();

    let rx = child_receiver.on.subscribe();

    thread::spawn(move || {
        while let Ok((v, reply)) = rx.recv() {
            println!("[Host] Received: {}", v);
            reply.send(v).unwrap()
        }
    });

    let response = child_sender.send_blocking(42);
    println!("[Host] Response: {}", response);
}
