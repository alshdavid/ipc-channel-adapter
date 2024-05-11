use std::thread;

use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use ipc_channel::ipc::channel as ipc_channel;
use ipc_channel::ipc::IpcReceiver;
use ipc_channel::ipc::IpcSender;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn create_ipc_child<TWrite, TRead>(
  host_server_name: &str,
) -> Result<(UnboundedSender<TWrite>, UnboundedReceiver<TRead>), ()>
where
  TWrite: Clone + Send + Serialize + DeserializeOwned + 'static,
  TRead: Clone + Send + Serialize + DeserializeOwned + 'static,
{
  // Proxies
  let (tx_child_outgoing, mut rx_child_outgoing) = unbounded_channel::<TWrite>();
  let (tx_child_incoming, rx_child_incoming) = unbounded_channel::<TRead>();

  let (itx_child_outgoing, irx_child_outgoing) = ipc_channel::<TWrite>().unwrap();
  let (itx_child_incoming, irx_child_incoming) = ipc_channel::<TRead>().unwrap();

  let Ok(child_outgoing_init) =
    IpcSender::<(IpcReceiver<TWrite>, IpcSender<TRead>)>::connect(host_server_name.to_string())
  else {
    return Err(());
  };

  // Proxy outgoing
  tokio::spawn(async move {
    while let Some(data) = rx_child_outgoing.recv().await {
      itx_child_outgoing.send(data).unwrap();
    }
  });

  // Proxy incoming, placed on a dedicated thread to avoid blocking
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
