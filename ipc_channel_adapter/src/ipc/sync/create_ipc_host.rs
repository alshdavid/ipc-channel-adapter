use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::fmt::Debug;

use ipc_channel::ipc::IpcOneShotServer;
use ipc_channel::ipc::IpcReceiver;
use ipc_channel::ipc::IpcSender;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn create_ipc_host<TWrite, TRead>() -> Result<(String, Sender<TWrite>, Receiver<TRead>), ()>
where
  TWrite: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
  TRead: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
{
  // Proxies
  let (tx_child_incoming, rx_child_incoming) = channel::<TRead>();
  let (tx_child_outgoing, rx_child_outgoing) = channel::<TWrite>();

  // Create a one shot channel that receives the "outgoing" and "incoming" channels
  let (child_incoming_init, child_incoming_server_name) =
    IpcOneShotServer::<(IpcReceiver<TRead>, IpcSender<TWrite>)>::new().unwrap();

  let (itx2, irx2) = channel::<IpcSender<TWrite>>();

  // Proxy outgoing
  thread::spawn(move || {
    let Ok(itx_child_outgoing) = irx2.recv() else {
      return;
    };

    while let Ok(data) = rx_child_outgoing.recv() {
      itx_child_outgoing.send(data).unwrap();
    }
  });

  thread::spawn(move || {
    // Receive the "outgoing" and "incoming" channels
    let Ok((_, (itx_child_incoming, itx_child_outgoing))) = child_incoming_init.accept() else {
      return;
    };

    if !itx2.send(itx_child_outgoing).is_ok() {
      return;
    };

    // Proxy incoming
    while let Ok(data) = itx_child_incoming.recv() {
      tx_child_incoming.send(data).unwrap();
    }
  });

  Ok((
    child_incoming_server_name,
    tx_child_outgoing,
    rx_child_incoming,
  ))
}
