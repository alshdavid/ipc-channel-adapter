use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SendError;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::context::IpcClientRequestContext;
use crate::context::IpcClientResponseContext;
use crate::ipc::sync::create_ipc_host;

pub struct ChildSender<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
{
  pub server_name: String,
  counter: Arc<AtomicUsize>,
  messages: Arc<Mutex<HashMap<usize, Sender<Response>>>>,
  tx_ipc: Sender<IpcClientRequestContext<Request>>,
}

impl<Request, Response> ChildSender<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
{
  pub fn new() -> Result<Self, String> {
    let (server_name, tx_ipc, rx_ipc) =
      create_ipc_host::<IpcClientRequestContext<Request>, IpcClientResponseContext<Response>>()?;

    let messages = Arc::new(Mutex::new(HashMap::<usize, Sender<Response>>::new()));

    thread::spawn({
      let messages = messages.clone();

      move || {
        while let Ok(data) = rx_ipc.recv() {
          let Some(sender) = messages.lock().unwrap().remove(&data.0) else {
            panic!("IPC Host: Can not find return signal");
          };
          sender.send(data.1).unwrap();
        }
      }
    });

    Ok(Self {
      server_name,
      messages,
      counter: Arc::new(AtomicUsize::new(0)),
      tx_ipc,
    })
  }

  pub fn send_blocking(&self, req: Request) -> Result<Response, String> {
    let Ok(response) = self.send(req) else {
      return Err(format!("IPC Host: Unable to send request"))
    };
    let Ok(response) = response.recv() else {
      return Err(format!("IPC Host: Unable to get response"))
    };
    return Ok(response)
  }

  pub fn send(&self, req: Request) -> Result<Receiver<Response>, SendError<Request>> {
    let count = self.counter.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = channel::<Response>();
    self.messages.lock().unwrap().insert(count.clone(), tx);
    if let Err(req) = self
      .tx_ipc
      .send(IpcClientRequestContext::<Request>(count.clone(), req)) {
        return Err(SendError(req.0.1));
      }
    Ok(rx)
  }
}
