use std::sync::Arc;

use crate::{
  api::core_api::raft::{
    raft_core_server::RaftCore, EntryResponse, HeartbeatRequest, VoteRequest,
    VoteResponse,
  },
  node::{
    entry::{Entry, Vote},
    node::RaftNode,
  },
};

use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

pub mod raft {
  tonic::include_proto!("raft");

  pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("raft_descriptor");
}

#[derive(Debug, Clone)]
pub struct RaftCoreService {
  node: Arc<Mutex<RaftNode>>,
}

impl RaftCoreService {
  pub fn new(node: Arc<Mutex<RaftNode>>) -> RaftCoreService {
    RaftCoreService {
      node,
    }
  }
}

#[tonic::async_trait]
impl RaftCore for RaftCoreService {
  async fn vote(
    &self,
    request: Request<VoteRequest>,
  ) -> Result<Response<VoteResponse>, Status> {
    let vote_request = request.get_ref();
    let candidate_id = vote_request.candidate_id as u16;
    let a_vote = Vote::new(vote_request.term, candidate_id);

    let mut node_lock = self.node.lock().await;
    let vote_result = node_lock.answer_vote(a_vote);
    drop(node_lock);

    let vote_response = VoteResponse {
      term: vote_result.vote_term(),
      vote_granted: vote_result.vote_was_granted(),
    };
    Ok(Response::new(vote_response))
  }

  async fn heartbeat(
    &self,
    request: Request<HeartbeatRequest>,
  ) -> Result<Response<EntryResponse>, Status> {
    let heartbeat_request = request.get_ref().clone();
    let the_message = heartbeat_request.message.unwrap();
    let a_heartbeat = Entry::new_heartbeat(
      the_message.term,
      the_message.leader_id as u16,
      the_message.prev_log_index,
      the_message.prev_log_term,
      the_message.leader_commit,
    );

    let mut node_lock = self.node.lock().await;
    let ack = node_lock.ack_heartbeat(a_heartbeat);
    drop(node_lock);

    let ack_response = EntryResponse {
      term: ack.entry_term(),
      success: ack.entry_success(),
    };
    Ok(Response::new(ack_response))
  }
}
