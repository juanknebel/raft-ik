use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::api::api_proto::raft::entry_point_server::EntryPointServer;
use raft::{
  entry_point_server::EntryPoint, health_server::Health,
  raft_core_server::RaftCore, CommandRequest, CommandResponse, EmptyRequest,
  EntryResponse, HeartbeatRequest, InfoResponse, VoteRequest, VoteResponse,
};

use crate::node::{
  entry::{Entry, Vote},
  node::RaftNode,
};

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

#[derive(Debug, Clone)]
pub struct HealthService {
  node: Arc<Mutex<RaftNode>>,
}

impl HealthService {
  pub fn new(node: Arc<Mutex<RaftNode>>) -> HealthService {
    HealthService {
      node,
    }
  }
}

#[tonic::async_trait]
impl Health for HealthService {
  async fn info(
    &self,
    _: Request<EmptyRequest>,
  ) -> Result<Response<InfoResponse>, Status> {
    let node_lock = self.node.lock().await;
    let response = InfoResponse {
      id: node_lock.id_node() as u32,
      term: node_lock.current_term(),
      leader: node_lock.leader().unwrap_or_default() as u32,
      vote_for: node_lock.vote_for().unwrap_or_default() as u32,
    };
    drop(node_lock);
    Ok(Response::new(response))
  }
}

#[derive(Debug, Clone)]
pub struct EntryPointService {
  node: Arc<Mutex<RaftNode>>,
}

impl EntryPointService {
  pub fn new(node: Arc<Mutex<RaftNode>>) -> EntryPointService {
    EntryPointService {
      node,
    }
  }
}

#[tonic::async_trait]
impl EntryPoint for EntryPointService {
  async fn command(
    &self,
    request: Request<CommandRequest>,
  ) -> Result<Response<CommandResponse>, Status> {
    let command_request = request.get_ref().clone();

    let mut node_lock = self.node.lock().await;
    let result = node_lock.handle_command(command_request.command);

    return match result {
      Ok(_) => {
        let command_response = CommandResponse {
          leader: node_lock.leader().unwrap_or_default() as u32,
          term: node_lock.current_term(),
        };
        drop(node_lock);
        Ok(Response::new(command_response))
      },
      Err(e) => {
        drop(node_lock);
        match e {
          None => Err(Status::internal("Unexpected error")),
          Some(address) => {
            let error_msg = format!(
              "I am a follower. Please send message to the leader {}",
              address.to_string()
            );
            Err(Status::permission_denied(error_msg))
          },
        }
      },
    };
  }
}
