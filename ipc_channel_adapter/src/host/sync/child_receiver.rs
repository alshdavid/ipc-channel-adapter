use std::marker::PhantomData;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::context::IpcClientRequestContext;
use crate::context::IpcClientResponseContext;
use crate::ipc::sync::create_ipc_host;

pub struct ChildReceiver<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + 'static,
{
  pub server_name: String,
  _0: PhantomData<Request>,
  _1: PhantomData<Response>,
}

impl<Request, Response> ChildReceiver<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + 'static,
{
  pub fn new() -> Result<(Self, Receiver<(Request, Sender<Response>)>), ()> {
    let (server_name, tx_ipc, rx_ipc) =
      create_ipc_host::<IpcClientResponseContext<Response>, IpcClientRequestContext<Request>>()
        .unwrap();

    let (tx, rx): (
      Sender<(Request, Sender<Response>)>,
      Receiver<(Request, Sender<Response>)>,
    ) = channel::<(Request, Sender<Response>)>();

    let tx1 = tx.clone();
    thread::spawn(move || {
      while let Ok(data) = rx_ipc.recv() {
        let (tx_reply, rx_reply) = channel::<Response>();
        tx1.send((data.1, tx_reply)).unwrap();
        let response = rx_reply.recv().unwrap();
        tx_ipc
          .send(IpcClientResponseContext::<Response>(data.0, response))
          .unwrap();
      }
    });

    Ok((
      Self {
        server_name,
        _0: PhantomData{},
        _1: PhantomData{},
      },
      rx,
    ))
  }

}
