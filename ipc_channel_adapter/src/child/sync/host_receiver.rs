use std::marker::PhantomData;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::context::IpcClientRequestContext;
use crate::context::IpcClientResponseContext;
use crate::ipc::sync::create_ipc_child;

pub struct HostReceiver<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
{
  _0: PhantomData<Request>,
  _1: PhantomData<Response>,
}

impl<Request, Response> HostReceiver<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
{
  pub fn new(channel_name: &str) -> Result<(Self, Receiver<(Request, Sender<Response>)>), String> {
    let ipc_child_client = channel_name.to_string();
    let (tx, rx) = channel::<(Request, Sender<Response>)>();

    let (tx_ipc, rx_ipc) = create_ipc_child::<
      IpcClientResponseContext<Response>,
      IpcClientRequestContext<Request>,
    >(&ipc_child_client)?;

    thread::spawn(move || {
      while let Ok(data) = rx_ipc.recv() {
        let (tx_reply, rx_reply) = channel::<Response>();
        if tx.send((data.1, tx_reply)).is_err() {
          panic!("IPC Child: Can not forward request");
        };

        let Ok(response) = rx_reply.recv() else {
          panic!("IPC Child: Unable to receive response");
        };

        if tx_ipc
          .send(IpcClientResponseContext::<Response>(data.0, response))
          .is_err()
        {
          println!("IPC Child: Unable to send message through IPC channel");
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
