use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::fmt::Debug;

use ipc_channel::ipc::channel as ipc_channel;
use ipc_channel::ipc::IpcReceiver;
use ipc_channel::ipc::IpcSender;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn create_ipc_child<TWrite, TRead>(
  host_server_name: &str,
) -> Result<(Sender<TWrite>, Receiver<TRead>), String>
where
  TWrite: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
  TRead: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
{
  // Proxies
  let (tx_child_outgoing, rx_child_outgoing) = channel::<TWrite>();
  let (tx_child_incoming, rx_child_incoming) = channel::<TRead>();

  let Ok((itx_child_outgoing, irx_child_outgoing)) = ipc_channel::<TWrite>() else {
    return Err(format!("IPC Child: Unable to create channel"));
  };

  let Ok((itx_child_incoming, irx_child_incoming)) = ipc_channel::<TRead>() else {
    return Err(format!("IPC Child: Unable to connect to create channel"));
  };

  let Ok(child_outgoing_init) =
    IpcSender::<(IpcReceiver<TWrite>, IpcSender<TRead>)>::connect(host_server_name.to_string()) else {
    return Err(format!("IPC Child: Unable to connect to host receiver"));
  };

  // Proxy outgoing
  thread::spawn(move || {
    while let Ok(data) = rx_child_outgoing.recv() {
      if itx_child_outgoing.send(data).is_err() {
        println!("Child Outgoing Send Error");
        panic!("Child Outgoing Send Error");
      };
    }
  });

  // Proxy incoming
  thread::spawn(move || {
    while let Ok(data) = irx_child_incoming.recv() {
      if tx_child_incoming.send(data).is_err() {
        println!("Child Incoming Send Error");
        panic!("Child Incoming Send Error");
      };
    }
  });

  if child_outgoing_init
    .send((irx_child_outgoing, itx_child_incoming))
    .is_err() {
    return Err(format!("IPC Child: Unable to establish handshake"));
  };

  Ok((tx_child_outgoing, rx_child_incoming))
}
