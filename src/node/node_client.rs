use crate::node::entry::{Entry, EntryResult, Vote, VoteResult};
use axum::async_trait;
use reqwest::{IntoUrl, Url};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Duration};

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
