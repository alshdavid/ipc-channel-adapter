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
) -> Result<(Sender<TWrite>, Receiver<TRead>), ()>
where
  TWrite: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
  TRead: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
{
  // Proxies
  let (tx_child_outgoing, rx_child_outgoing) = channel::<TWrite>();
  let (tx_child_incoming, rx_child_incoming) = channel::<TRead>();

  let (itx_child_outgoing, irx_child_outgoing) = ipc_channel::<TWrite>().unwrap();
  let (itx_child_incoming, irx_child_incoming) = ipc_channel::<TRead>().unwrap();

  // Receive a one shot channel to send back the "outgoing" and "incoming" channels
  let Ok(child_outgoing_init) =
    IpcSender::<(IpcReceiver<TWrite>, IpcSender<TRead>)>::connect(host_server_name.to_string())
  else {
    return Err(());
  };

  // Proxy outgoing
  thread::spawn(move || {
    while let Ok(data) = rx_child_outgoing.recv() {
      itx_child_outgoing.send(data).unwrap();
    }
  });

  // Proxy incoming
  thread::spawn(move || {
    while let Ok(data) = irx_child_incoming.recv() {
      tx_child_incoming.send(data).unwrap();
    }
  });

  child_outgoing_init
    .send((irx_child_outgoing, itx_child_incoming))
    .unwrap();

  Ok((tx_child_outgoing, rx_child_incoming))
}
