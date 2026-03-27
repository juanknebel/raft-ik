use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use raft_api::{
  entry_point_server::EntryPoint, health_server::Health, CommandRequest,
  CommandResponse, EmptyRequest, InfoResponse,
};

use crate::node::node::RaftNode;

pub mod raft_api {
  tonic::include_proto!("raft_api");

  pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("raft_api_descriptor");
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
