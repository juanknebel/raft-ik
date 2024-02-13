use crate::{
  error::{Error, Result},
  node::entry::RaftEntryResponse,
};
use std::sync::{Arc, Mutex};

use axum::{
  extract::{rejection::JsonRejection, State},
  routing::post,
  Json, Router,
};

use log::info;

use crate::node::{entry::RaftEntry, node::RaftNode};

pub fn routes(node: Arc<Mutex<RaftNode>>) -> Router {
  info!("Adding POST /heartbeat");
  Router::new()
    .route("/heartbeat", post(heartbeat))
    .with_state(node)
}

pub async fn heartbeat(
  State(node): State<Arc<Mutex<RaftNode>>>,
  heartbeat_param: core::result::Result<Json<RaftEntry>, JsonRejection>,
) -> Result<Json<RaftEntryResponse>> {
  info!("Processing hearbeat");
  let Json(heartbeat) = heartbeat_param.map_err(|e| Error::ParseError {
    kind: e.body_text(),
  })?;
  let mut node_unlock = node.lock().map_err(|e| Error::LockError)?;
  let response = node_unlock.process(heartbeat);
  Ok(Json(response))
}
