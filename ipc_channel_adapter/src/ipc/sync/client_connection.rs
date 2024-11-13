use std::fmt::Debug;
use std::io;

use ipc_channel::ipc::channel as ipc_channel;
use ipc_channel::ipc::IpcReceiver;
use ipc_channel::ipc::IpcSender;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct ClientConnection<TWrite, TRead>
where
  TWrite: Send + Serialize + DeserializeOwned + Debug + 'static,
  TRead: Send + Serialize + DeserializeOwned + Debug + 'static,
{
  rx: IpcReceiver<(TWrite, IpcSender<TRead>)>,
}

impl<TWrite, TRead> ClientConnection<TWrite, TRead>
where
  TWrite: Send + Serialize + DeserializeOwned + Debug + 'static,
  TRead: Send + Serialize + DeserializeOwned + Debug + 'static,
{
  pub fn new<S: AsRef<str>>(server_name: S) -> Result<Self, String> {
    let Ok((tx, rx)) = ipc_channel::<(TWrite, IpcSender<TRead>)>() else {
      return Err(format!("IPC Child: Unable to connect to create channel"));
    };

    let Ok(child_outgoing_init) =
      IpcSender::<IpcSender<(TWrite, IpcSender<TRead>)>>::connect(server_name.as_ref().to_string())
    else {
      return Err(format!("IPC Child: Unable to connect to host receiver"));
    };

    child_outgoing_init.send(tx).unwrap();
    
    Ok(Self {
      rx
    })
  }

  pub fn recv(&self) -> Result<(TWrite, IpcSender<TRead>), io::Error> {
    match self.rx.recv() {
      Ok(msg) => Ok(msg),
      Err(err) => Err(io::Error::other(err)),
    }
  }
}
