use serde::Serialize;
use serde::Deserialize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IpcClientRequestContext<T>(pub usize, pub T);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IpcClientResponseContext<T>(pub usize, pub T);
