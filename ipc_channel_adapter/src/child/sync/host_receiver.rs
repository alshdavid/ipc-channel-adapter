use std::marker::PhantomData;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::context::IpcClientRequestContext;
use crate::context::IpcClientResponseContext;
use crate::ipc::sync::create_ipc_child;

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
  pub fn new(channel_name: &str) -> Result<(Self, Receiver<(Request, Sender<Response>)>), ()> {
    let ipc_child_client = channel_name.to_string();
    let (tx, rx) = channel::<(Request, Sender<Response>)>();

    thread::spawn(move || {
      let Ok((tx_ipc, rx_ipc)) = create_ipc_child::<
        IpcClientResponseContext<Response>,
        IpcClientRequestContext<Request>,
      >(&ipc_child_client) else {
        return;
      };

      while let Ok(data) = rx_ipc.recv() {
        let (tx_reply, rx_reply) = channel::<Response>();
        tx.send((data.1, tx_reply)).unwrap();
        let response = rx_reply.recv().unwrap();
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
