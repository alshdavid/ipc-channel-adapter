use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::Mutex;

use crate::kit::ipc::asynch::IpcHost;
use crate::public::nodejs::NodejsClientRequest;
use crate::public::nodejs::NodejsClientRequestContext;
use crate::public::nodejs::NodejsClientResponse;
use crate::public::nodejs::NodejsClientResponseContext;

#[derive(Clone)]
pub struct ChildSender {
  pub server_name: String,
  counter: Arc<AtomicUsize>,
  messages: Arc<Mutex<HashMap<usize, UnboundedSender<NodejsClientResponse>>>>,
  ipc_host_client: IpcHost<NodejsClientRequestContext, NodejsClientResponseContext>
}

impl ChildSender {
  pub fn new() -> Self {
    let ipc_host_client = IpcHost::<NodejsClientRequestContext, NodejsClientResponseContext>::new();
    let server_name = ipc_host_client.server_name.clone();

    let messages = Arc::new(Mutex::new(
      HashMap::<usize, UnboundedSender<NodejsClientResponse>>::new(),
    ));

    {
      let mut rx = ipc_host_client.subscribe();
      let messages = messages.clone();
      
      tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
          let Some(sender) = messages.lock().await.remove(&data.0) else {
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

  pub async fn send_blocking(&self, req: NodejsClientRequest) -> NodejsClientResponse {
    self.send(req).await.recv().await.unwrap()
  }

  pub async fn send(&self, req: NodejsClientRequest) -> UnboundedReceiver<NodejsClientResponse> {
    let count = self.counter.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = unbounded_channel::<NodejsClientResponse>();
    self.messages.lock().await.insert(count.clone(), tx);
    self.ipc_host_client.send(NodejsClientRequestContext(
      count.clone(),
      req,
    ));
    rx
  }
}
