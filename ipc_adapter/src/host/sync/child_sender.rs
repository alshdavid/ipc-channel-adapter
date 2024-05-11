use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::ipc::sync::IpcHost;
use crate::context::IpcClientRequestContext;
use crate::context::IpcClientResponseContext;

#[derive(Clone)]
pub struct ChildSender <Request, Response> 
  where 
    Request: Clone + Send + Serialize + DeserializeOwned + 'static,
    Response: Clone + Send + Serialize + DeserializeOwned + 'static {
  pub server_name: String,
  counter: Arc<AtomicUsize>,
  messages: Arc<Mutex<HashMap<usize, Sender<Response>>>>,
  ipc_host_client: IpcHost<IpcClientRequestContext<Request>, IpcClientResponseContext<Response>>
}

impl<Request, Response> ChildSender<Request, Response> 
  where 
    Request: Clone + Send + Serialize + DeserializeOwned + 'static,
    Response: Clone + Send + Serialize + DeserializeOwned + 'static {
  pub fn new() -> Self {
    let ipc_host_client = IpcHost::<IpcClientRequestContext<Request>, IpcClientResponseContext<Response>>::new();
    let server_name = ipc_host_client.server_name.clone();

    let messages = Arc::new(Mutex::new(
      HashMap::<usize, Sender<Response>>::new(),
    ));

    {
      let rx = ipc_host_client.subscribe();
      let messages = messages.clone();
      
      thread::spawn(move || {
        while let Ok(data) = rx.recv() {
          let Some(sender) = messages.lock().unwrap().remove(&data.0) else {
            panic!();
          };
          sender.send(data.1).unwrap();
        }
      });
    }

    Self {
      server_name,
      messages,
      counter: Arc::new(AtomicUsize::new(0)),
      ipc_host_client,
    }
  }

  pub fn send_blocking(&self, req: Request) -> Response {
    self.send(req).recv().unwrap()
  }

  pub fn send(&self, req: Request) -> Receiver<Response> {
    let count = self.counter.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = channel::<Response>();
    self.messages.lock().unwrap().insert(count.clone(), tx);
    self.ipc_host_client.send(IpcClientRequestContext::<Request>(
      count.clone(),
      req,
    ));
    rx
  }
}
