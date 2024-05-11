use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::broadcast_channel::sync::BroadcastChannel;
use crate::context::IpcClientRequestContext;
use crate::context::IpcClientResponseContext;
use crate::ipc::sync::IpcChild;

pub struct HostReceiver<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + 'static,
{
  pub on: BroadcastChannel<(Request, Sender<Response>)>,
}

impl<Request, Response> HostReceiver<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + 'static,
{
  pub fn new(channel_name: &str) -> Self {
    let ipc_child_client = channel_name.to_string();
    let trx = BroadcastChannel::<(Request, Sender<Response>)>::new();

    let tx = trx.clone();
    thread::spawn(move || {
      let Ok(ipc_child_client) = IpcChild::<
        IpcClientResponseContext<Response>,
        IpcClientRequestContext<Request>,
      >::new(&ipc_child_client) else {
        return;
      };
      let irx = ipc_child_client.subscribe();

      while let Ok(data) = irx.recv() {
        match data.1 {
          req => {
            let (tx_reply, rx_reply) = channel::<Response>();
            tx.send((req, tx_reply)).unwrap();
            let response = rx_reply.recv().unwrap();
            ipc_child_client.send(IpcClientResponseContext::<Response>(data.0, response));
          }
        }
      }
    });

    Self { on: trx }
  }

  pub fn subscribe(&self) -> Receiver<(Request, Sender<Response>)> {
    self.on.subscribe()
  }
}
