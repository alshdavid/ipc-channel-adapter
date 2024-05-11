use ipc_channel::ipc::IpcOneShotServer;
use ipc_channel::ipc::IpcReceiver;
use ipc_channel::ipc::IpcSender;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::thread;

use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone)]
pub struct IpcHost<TWrite, TRead>
where
  TWrite: Clone + Send + Serialize + DeserializeOwned + 'static,
  TRead: Clone + Send + Serialize + DeserializeOwned + 'static,
{
  pub server_name: String,
  outgoing: UnboundedSender<TWrite>,
  incoming: UnboundedSender<UnboundedSender<TRead>>,
}

impl<TWrite, TRead> IpcHost<TWrite, TRead>
where
  TWrite: Clone + Send + Serialize + DeserializeOwned + 'static,
  TRead: Clone + Send + Serialize + DeserializeOwned + 'static,
{
  pub fn new() -> Self {
    // Proxies
    let (tx_child_incoming, mut rx_child_incoming) = unbounded_channel::<UnboundedSender<TRead>>();
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
      let mut senders = Vec::<Option<UnboundedSender<TRead>>>::new();

      while let Ok(data) = itx_child_incoming.recv() {
        for sender_opt in senders.iter_mut() {
          let Some(sender) = sender_opt else {
            continue;
          };
          if sender.send(data.clone()).is_err() {
            sender_opt.take();
          }
        }
        while let Ok(sender) = rx_child_incoming.try_recv() {
          if sender.send(data.clone()).is_ok() {
            senders.push(Some(sender));
          }
        }
      }
    });

    Self {
      server_name: child_incoming_server_name,
      outgoing: tx_child_outgoing,
      incoming: tx_child_incoming,
    }
  }

  pub fn send(
    &self,
    data: TWrite,
  ) {
    self.outgoing.send(data).unwrap();
  }

  pub fn subscribe(&self) -> UnboundedReceiver<TRead> {
    let (tx, rx) = unbounded_channel::<TRead>();
    self.incoming.send(tx).unwrap();
    rx
  }
}
