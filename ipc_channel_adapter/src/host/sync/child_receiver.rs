use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::broadcast_channel::sync::BroadcastChannel;
use crate::context::IpcClientRequestContext;
use crate::context::IpcClientResponseContext;
use crate::ipc::sync::IpcHost;

#[derive(Clone)]
pub struct ChildReceiver<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + 'static,
{
  pub server_name: String,
  pub on: BroadcastChannel<(Request, Sender<Response>)>,
}

impl<Request, Response> ChildReceiver<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + 'static,
{
  pub fn new() -> Self {
    let ipc_host_host =
      IpcHost::<IpcClientResponseContext<Response>, IpcClientRequestContext<Request>>::new();
    let server_name = ipc_host_host.server_name.clone();

    let trx = BroadcastChannel::<(Request, Sender<Response>)>::new();

    {
      let irx = ipc_host_host.subscribe();
      let trx = trx.clone();
      thread::spawn(move || {
        while let Ok(data) = irx.recv() {
          match data.1 {
            req => {
              let (tx_reply, rx_reply) = channel::<Response>();
              trx.send((req, tx_reply)).unwrap();
              let response = rx_reply.recv().unwrap();
              ipc_host_host.send(IpcClientResponseContext::<Response>(data.0, response));
            }
          }
        }
      });
    }

    Self {
      server_name,
      on: trx,
    }
  }
}
