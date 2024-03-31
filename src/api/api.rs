use crate::{
  error::{Error, Result},
  node::entry::{RaftEntryResponse, RaftRequestVote, RaftVoteResponse},
};
use std::sync::Arc;

use axum::{
  extract::{rejection::JsonRejection, State},
  routing::{get, post},
  Json, Router,
};

use log::info;
use serde::Serialize;
use tokio::sync::Mutex;

use crate::node::{entry::RaftEntry, node::RaftNode};

pub fn routes(node: Arc<Mutex<RaftNode>>) -> Router {
  info!("Adding POST /heartbeat");
  info!("Adding POST /vote");
  info!("Adding GET /info");
  Router::new()
    .route("/heartbeat", post(heartbeat))
    .route("/vote", post(vote))
    .route("/info", get(info))
    .with_state(node)
}

pub async fn heartbeat(
  State(node): State<Arc<Mutex<RaftNode>>>,
  heartbeat_param: core::result::Result<Json<RaftEntry>, JsonRejection>,
) -> Result<Json<RaftEntryResponse>> {
  let Json(heartbeat) = heartbeat_param.map_err(|e| Error::ParseError {
    kind: e.body_text(),
  })?;
  let mut node_lock = node.lock().await;
  let response = node_lock.ack_heartbeat(heartbeat);
  drop(node_lock);
  Ok(Json(response))
}

pub async fn vote(
  State(node): State<Arc<Mutex<RaftNode>>>,
  vote_param: core::result::Result<Json<RaftRequestVote>, JsonRejection>,
) -> Result<Json<RaftVoteResponse>> {
  let Json(vote) = vote_param.map_err(|e| Error::ParseError {
    kind: e.body_text(),
  })?;
  let mut node_lock = node.lock().await;
  let response = node_lock.answer_vote(vote);
  drop(node_lock);
  Ok(Json(response))
}

pub async fn info(
  State(node): State<Arc<Mutex<RaftNode>>>,
) -> Result<Json<RaftInfo>> {
  let node_lock = node.lock().await;
  let response = RaftInfo {
    id: node_lock.id_node(),
    term: node_lock.current_term(),
    leader: node_lock.leader(),
    vote_for: node_lock.vote_for(),
  };
  drop(node_lock);
  Ok(Json(response))
}

#[derive(Serialize)]
struct RaftInfo {
  id: u16,
  term: u64,
  leader: Option<u16>,
  vote_for: Option<u16>,
}
