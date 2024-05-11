use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot::channel as oneshot_channel;
use tokio::sync::oneshot::Receiver as OneshotReceiver;
use tokio::sync::oneshot::Sender as OneshotSender;
use tokio::sync::Mutex;

use crate::context::IpcClientRequestContext;
use crate::context::IpcClientResponseContext;
use crate::ipc::asynch::create_ipc_host;

pub struct ChildSender<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
{
  pub server_name: String,
  counter: Arc<AtomicUsize>,
  messages: Arc<Mutex<HashMap<usize, OneshotSender<Response>>>>,
  tx_ipc: UnboundedSender<IpcClientRequestContext<Request>>,
}

impl<Request, Response> ChildSender<Request, Response>
where
  Request: Clone + Send + Serialize + DeserializeOwned + 'static,
  Response: Clone + Send + Serialize + DeserializeOwned + Debug + 'static,
{
  pub fn new() -> Self {
    let (server_name, tx_ipc, mut rx_ipc) =
      create_ipc_host::<IpcClientRequestContext<Request>, IpcClientResponseContext<Response>>()
        .unwrap();

    let messages = Arc::new(Mutex::new(HashMap::<usize, OneshotSender<Response>>::new()));

    tokio::spawn({
      let messages = messages.clone();

      async move {
        while let Some(data) = rx_ipc.recv().await {
          let Some(sender) = messages.lock().await.remove(&data.0) else {
            panic!();
          };
          sender.send(data.1).unwrap();
        }
      }
    });

    Self {
      server_name,
      messages,
      counter: Arc::new(AtomicUsize::new(0)),
      tx_ipc,
    }
  }

  pub async fn send_and_wait(&self, req: Request) -> Response {
    self.send(req).await.await.unwrap()
  }

  pub async fn send(&self, req: Request) -> OneshotReceiver<Response> {
    let count = self.counter.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = oneshot_channel::<Response>();
    self.messages.lock().await.insert(count.clone(), tx);
    self
      .tx_ipc
      .send(IpcClientRequestContext::<Request>(count.clone(), req))
      .unwrap();
    rx
  }
}
