use std::marker::PhantomData;
use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::oneshot::channel as oneshot_channel;
use tokio::sync::oneshot::Sender as OneshotSender;

use crate::context::IpcClientRequestContext;
use crate::context::IpcClientResponseContext;
use crate::ipc::asynch::create_ipc_host;

pub struct ChildReceiver<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
{
  pub server_name: String,
  _0: PhantomData<Request>,
  _1: PhantomData<Response>,
}

impl<Request, Response> ChildReceiver<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
{
  pub fn new() -> Result<(Self, UnboundedReceiver<(Request, OneshotSender<Response>)>), ()> {
    let (server_name, tx_ipc, mut rx_ipc) =
      create_ipc_host::<IpcClientResponseContext<Response>, IpcClientRequestContext<Request>>()
        .unwrap();

    let (tx, rx) = unbounded_channel::<(Request, OneshotSender<Response>)>();

    tokio::spawn({
      let tx = tx.clone();

      async move {
        while let Some(data) = rx_ipc.recv().await {
          let (tx_reply, rx_reply) = oneshot_channel::<Response>();
          tx.send((data.1, tx_reply)).unwrap();
          let response = rx_reply.await.unwrap();
          tx_ipc
            .send(IpcClientResponseContext::<Response>(data.0, response))
            .unwrap();
        }
      }
    });

    Ok((
      Self {
        server_name,
        _0: PhantomData {},
        _1: PhantomData {},
      },
      rx,
    ))
  }
}
