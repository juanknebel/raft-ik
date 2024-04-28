use std::{net::SocketAddr, time::Duration};

use axum::async_trait;
use tonic::transport::Channel;

use crate::{
  api::core_api::raft::{
    raft_core_client::RaftCoreClient, HeartbeatRequest, RaftMessage,
    VoteRequest,
  },
  node::entry::{Entry, EntryResult, Vote, VoteResult},
};

#[async_trait]
pub trait NodeClient: Send + Sync + 'static {
  async fn request_for_vote(
    &self,
    address: &SocketAddr,
    a_vote: &Vote,
  ) -> Result<VoteResult, String>;
  async fn send_heartbeat(
    &self,
    address: &SocketAddr,
    a_heartbeat: &Entry,
  ) -> Result<EntryResult, String>;
}

impl std::fmt::Debug for dyn NodeClient {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "dyn NodeClient")
  }
}

#[derive(Debug)]
pub struct RpcNodeClient {
  timeout: Duration,
}

impl RpcNodeClient {
  pub fn new(timeout: Duration) -> RpcNodeClient {
    RpcNodeClient {
      timeout,
    }
  }

  pub async fn create_client(
    &self,
    address: &SocketAddr,
  ) -> Result<RaftCoreClient<Channel>, String> {
    let url = format!("http://{}:{}", address.ip(), address.port());
    let client = RaftCoreClient::connect(url)
      .await
      .map_err(|e| e.to_string());
    client
  }
}

#[async_trait]
impl NodeClient for RpcNodeClient {
  async fn request_for_vote(
    &self,
    address: &SocketAddr,
    a_vote: &Vote,
  ) -> Result<VoteResult, String> {
    let mut client = self.create_client(address).await?;
    let vote_request = VoteRequest {
      term: a_vote.term(),
      candidate_id: a_vote.candidate() as u32,
      last_log_index: a_vote.log_index(),
      last_log_term: a_vote.log_term(),
    };
    let mut request = tonic::Request::new(vote_request);
    request.set_timeout(self.timeout);
    let response = client.vote(request).await.map_err(|e| e.to_string())?;
    if response.get_ref().vote_granted {
      Ok(VoteResult::success(response.get_ref().term))
    } else {
      Ok(VoteResult::failure(response.get_ref().term))
    }
  }

  async fn send_heartbeat(
    &self,
    address: &SocketAddr,
    a_heartbeat: &Entry,
  ) -> Result<EntryResult, String> {
    let mut client = self.create_client(address).await?;
    let heartbeat_request = HeartbeatRequest {
      message: Some(RaftMessage {
        term: a_heartbeat.term(),
        leader_id: a_heartbeat.leader_id() as u32,
        prev_log_index: 0,
        prev_log_term: 0,
        leader_commit: 0,
      }),
    };
    let mut request = tonic::Request::new(heartbeat_request);
    request.set_timeout(self.timeout);
    let response =
      client.heartbeat(request).await.map_err(|e| e.to_string())?;
    if response.get_ref().success {
      Ok(EntryResult::success(response.get_ref().term))
    } else {
      Ok(EntryResult::failure(response.get_ref().term))
    }
  }
}
