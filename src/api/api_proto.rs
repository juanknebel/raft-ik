use crate::node::node::RaftNode;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::node::entry::RaftRequestVote;

use raft::{
  command_server::Command, heartbeat_server::Heartbeat, info_server::Info,
  vote_server::Vote, CommandRequest, CommandResponse, HeartbeatRequest,
  HeartbeatResponse, InfoResponse, VoteRequest, VoteResponse,
};
pub mod raft {
  tonic::include_proto!("raft");
}

#[derive(Debug)]
pub struct VotingService {
  node: Arc<Mutex<RaftNode>>,
}

impl VotingService {
  pub fn new(node: Arc<Mutex<RaftNode>>) -> VotingService {
    VotingService {
      node,
    }
  }
}

#[tonic::async_trait]
impl Vote for VotingService {
  async fn vote(
    &self,
    request: Request<VoteRequest>,
  ) -> Result<Response<VoteResponse>, Status> {
    let vote_request = request.get_ref();
    let candidate_id = vote_request.candidate_id as u16;
    let a_vote = RaftRequestVote::new(vote_request.term, candidate_id);
    let mut node_lock = self.node.lock().await;
    let response = node_lock.answer_vote(a_vote);
    let vote_response = VoteResponse {
      term: response.vote_term(),
      vote_granted: response.vote_was_granted(),
    };
    Ok(Response::new(vote_response))
  }
}
