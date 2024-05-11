use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::UnboundedSender;

use crate::kit::broadcast_channel::asynch::BroadcastChannel;
use crate::kit::ipc::asynch::IpcHost;
use crate::public::nodejs::NodejsHostRequest;
use crate::public::nodejs::NodejsHostRequestContext;
use crate::public::nodejs::NodejsHostResponse;
use crate::public::nodejs::NodejsHostResponseContext;

#[derive(Clone)]
pub struct ChildReceiver {
  pub server_name: String,
  pub on: BroadcastChannel<(NodejsHostRequest, UnboundedSender<NodejsHostResponse>)>,
}

impl ChildReceiver {
  pub fn new() -> Self {
    let ipc_host_host = IpcHost::<NodejsHostResponseContext, NodejsHostRequestContext>::new();
    let server_name = ipc_host_host.server_name.clone();

    let trx = BroadcastChannel::<(NodejsHostRequest, UnboundedSender<NodejsHostResponse>)>::new();
    
    {
      let mut irx = ipc_host_host.subscribe();
      let trx = trx.clone();
      tokio::spawn(async move {
        while let Some(data) = irx.recv().await {
          match data.1 {
            req => {
              let (tx_reply, mut rx_reply) = unbounded_channel::<NodejsHostResponse>();
              trx.send((req, tx_reply)).unwrap();
              let response = rx_reply.recv().await.unwrap();
              ipc_host_host.send(NodejsHostResponseContext(data.0, response));
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
