use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use crate::context::IpcClientRequestContext;
use crate::context::IpcClientResponseContext;
use crate::ipc::asynch::create_ipc_child;

pub struct HostReceiver<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + 'static,
{
  _0: PhantomData<Request>,
  _1: PhantomData<Response>,
}

impl<Request, Response> HostReceiver<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + 'static,
{
  pub fn new(
    channel_name: &str,
  ) -> Result<
    (
      Self,
      UnboundedReceiver<(Request, UnboundedSender<Response>)>,
    ),
    (),
  > {
    let ipc_child_client = channel_name.to_string();
    let (tx, rx) = unbounded_channel::<(Request, UnboundedSender<Response>)>();

    tokio::spawn(async move {
      let Ok((tx_ipc, mut rx_ipc)) = create_ipc_child::<
        IpcClientResponseContext<Response>,
        IpcClientRequestContext<Request>,
      >(&ipc_child_client) else {
        return;
      };

      while let Some(data) = rx_ipc.recv().await {
        let (tx_reply, mut rx_reply) = unbounded_channel::<Response>();
        tx.send((data.1, tx_reply)).unwrap();
        let response = rx_reply.recv().await.unwrap();
        if tx_ipc
          .send(IpcClientResponseContext::<Response>(data.0, response))
          .is_err()
        {
          return;
        };
      }
    });

    Ok((
      Self {
        _0: PhantomData {},
        _1: PhantomData {},
      },
      rx,
    ))
  }
}
