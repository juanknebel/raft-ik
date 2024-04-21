use std::{net::SocketAddr, time::Duration};

use axum::async_trait;
use reqwest::{IntoUrl, Url};
use serde::{Deserialize, Serialize};

use crate::{
  api::api_proto::raft::{
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
pub struct HttpNodeClient {
  timeout: Duration,
  vote_url: String,
  heartbeat_url: String,
}

impl HttpNodeClient {
  pub fn new(timeout: Duration) -> Self {
    HttpNodeClient {
      timeout,
      vote_url: "vote".into(),
      heartbeat_url: "heartbeat".into(),
    }
  }

  async fn post_request<T: Serialize + ?Sized, E: for<'a> Deserialize<'a>>(
    &self,
    body: &T,
    url: impl IntoUrl,
  ) -> Result<E, String> {
    let client = reqwest::Client::builder()
      .timeout(self.timeout)
      .build()
      .map_err(|e| e.to_string())?;
    let response = client
      .post(url)
      .json(body)
      .send()
      .await
      .map_err(|e| e.to_string())?;
    response.json::<E>().await.map_err(|e| e.to_string())
  }
}

#[async_trait]
impl NodeClient for HttpNodeClient {
  async fn request_for_vote(
    &self,
    address: &SocketAddr,
    a_vote: &Vote,
  ) -> Result<VoteResult, String> {
    let url = format!(
      "http://{}:{}/{}",
      address.ip(),
      address.port(),
      self.vote_url
    );
    let request_url = Url::parse(&url).expect("Failed to parse URL");
    self.post_request(a_vote, request_url).await
  }

  async fn send_heartbeat(
    &self,
    address: &SocketAddr,
    a_heartbeat: &Entry,
  ) -> Result<EntryResult, String> {
    let url = format!(
      "http://{}:{}/{}",
      address.ip(),
      address.port(),
      self.heartbeat_url
    );
    let request_url = Url::parse(&url).expect("Failed to parse URL");
    self.post_request(a_heartbeat, request_url).await
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
}

#[async_trait]
impl NodeClient for RpcNodeClient {
  async fn request_for_vote(
    &self,
    address: &SocketAddr,
    a_vote: &Vote,
  ) -> Result<VoteResult, String> {
    let url = format!("http://{}:{}", address.ip(), address.port());
    let mut client = RaftCoreClient::connect(url)
      .await
      .map_err(|e| e.to_string())?;
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
    let url = format!("http://{}:{}", address.ip(), address.port());
    let mut client = RaftCoreClient::connect(url)
      .await
      .map_err(|e| e.to_string())?;
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
