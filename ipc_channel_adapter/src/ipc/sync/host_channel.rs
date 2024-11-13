use std::fmt::Debug;
use std::io;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;

use ipc_channel::ipc::IpcOneShotServer;
use ipc_channel::ipc::IpcSender;
use ipc_channel::ipc::channel as ipc_channel;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct HostConnection<TWrite, TRead>
where
  TWrite: Send + Serialize + DeserializeOwned + Debug + 'static,
  TRead: Send + Serialize + DeserializeOwned + Debug + 'static,
{
  tx: Sender<(TWrite, IpcSender<TRead>)>,
  server_name: String
}

impl<TWrite, TRead> HostConnection<TWrite, TRead>
where
  TWrite: Send + Serialize + DeserializeOwned + Debug + 'static,
  TRead: Send + Serialize + DeserializeOwned + Debug + 'static,
{
  pub fn new() -> Result<Self, String> {
    // Create a one shot channel that receives the "outgoing" and "incoming" channels
    let Ok((child_incoming_init, server_name)) =
      IpcOneShotServer::<IpcSender<(TWrite, IpcSender<TRead>)>>::new()
    else {
      return Err(format!("IPC Host: Unable to create handshake server"));
    };

    let (tx, rx) = channel();

    thread::spawn(move || {
      let Ok((_, tx_ipc)) = child_incoming_init.accept() else {
        println!("IPC Host: Unable to handshake");
        panic!("IPC Host: Unable to handshake");
      };

      while let Ok(msg) = rx.recv() {
        tx_ipc.send(msg).unwrap();
      }
    });

    Ok(Self { tx, server_name })
  }

  pub fn send(&self, msg: TWrite) -> Result<TRead, io::Error> {
    let (tx, rx) = ipc_channel()?;
    if let Err(_) = self.tx.send((msg, tx)) {
      return Err(io::Error::other("Failed"));
    }
    match rx.recv() {
        Ok(resp) => Ok(resp),
        Err(err) => Err(io::Error::other(err)),
    }
  }

  pub fn server_name(&self) -> &str {
    &self.server_name
  }
}
