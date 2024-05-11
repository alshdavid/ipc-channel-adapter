use std::thread;

use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use ipc_channel::ipc::IpcOneShotServer;
use ipc_channel::ipc::IpcReceiver;
use ipc_channel::ipc::IpcSender;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn create_ipc_host<TWrite, TRead>() -> Result<(String, UnboundedSender<TWrite>, UnboundedReceiver<TRead>), ()>
where
  TWrite: Clone + Send + Serialize + DeserializeOwned + 'static,
  TRead: Clone + Send + Serialize + DeserializeOwned + 'static,
{
  // Proxies
  let (tx_child_incoming, rx_child_incoming) = unbounded_channel::<TRead>();
  let (tx_child_outgoing, mut rx_child_outgoing) = unbounded_channel::<TWrite>();

  // Create a one shot channel that receives the "outgoing" and "incoming" channels
  let (child_incoming_init, child_incoming_server_name) =
    IpcOneShotServer::<(IpcReceiver<TRead>, IpcSender<TWrite>)>::new().unwrap();

  let (itx2, mut irx2) = unbounded_channel::<IpcSender<TWrite>>();

  // Proxy outgoing
  tokio::spawn(async move {
    let Some(itx_child_outgoing) = irx2.recv().await else {
      return;
    };

    while let Some(data) = rx_child_outgoing.recv().await {
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
