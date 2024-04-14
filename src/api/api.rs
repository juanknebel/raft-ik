use crate::{
  error::{Error, Result},
  node::entry::{EntryResult, Vote, VoteResult},
};
use std::sync::Arc;

use axum::{
  extract::{rejection::JsonRejection, State},
  http::StatusCode,
  response::{IntoResponse, Response},
  routing::{get, post},
  Json, Router,
};

use log::info;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::node::{entry::Entry, node::RaftNode};

pub fn routes(node: Arc<Mutex<RaftNode>>) -> Router {
  info!("Adding POST /heartbeat");
  info!("Adding POST /vote");
  info!("Adding GET /info");
  info!("Adding POST /command");
  Router::new()
    .route("/heartbeat", post(heartbeat))
    .route("/vote", post(vote))
    .route("/info", get(info))
    .route("/command", post(add_command))
    .with_state(node)
}

pub async fn heartbeat(
  State(node): State<Arc<Mutex<RaftNode>>>,
  heartbeat_param: core::result::Result<Json<Entry>, JsonRejection>,
) -> Result<Json<EntryResult>> {
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
  vote_param: core::result::Result<Json<Vote>, JsonRejection>,
) -> Result<Json<VoteResult>> {
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

pub async fn add_command(
  State(node): State<Arc<Mutex<RaftNode>>>,
  command_param: core::result::Result<Json<RaftCommand>, JsonRejection>,
) -> Response {
  let mut node_lock = node.lock().await;
  if command_param.is_err() {
    return (
      StatusCode::BAD_REQUEST,
      command_param.err().unwrap().body_text(),
    )
      .into_response();
  }
  let Json(a_command) = command_param.unwrap();
  return match node_lock.handle_command(a_command.some_command) {
    Ok(_) => {
      let response = RaftCommandResponse {
        leader: node_lock.leader().unwrap(),
        term: node_lock.current_term(),
      };
      (StatusCode::ACCEPTED, Json(response)).into_response()
    },
    Err(e) => match e {
      None => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Unexpected error",
      )
        .into_response(),
      Some(address) => (
        StatusCode::PERMANENT_REDIRECT,
        address.to_string(),
      )
        .into_response(),
    },
  };
}

#[derive(Serialize)]
pub struct RaftInfo {
  id: u16,
  term: u64,
  leader: Option<u16>,
  vote_for: Option<u16>,
}

#[derive(Deserialize)]
pub struct RaftCommand {
  some_command: String,
}

#[derive(Serialize)]
struct RaftCommandResponse {
  leader: u16,
  term: u64,
}
