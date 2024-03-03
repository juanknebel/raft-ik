use std::{
  collections::HashMap,
  net::SocketAddr,
  time::{Duration, Instant},
};

use log::{error, info};
use rand::Rng;

use crate::node::entry::RaftVoteResponse;

use super::{
  entry::{RaftEntry, RaftEntryResponse, RaftRequestVote},
  state::RaftState,
};

/// The RaftNode contains all the necessary information to perform all the actions to mantain
/// a consensus between the whole network of nodes in a distributed system.
/// The node at any given moment could be only one the following positions: Follower, Candidate,
/// Leader. Every time the node came to life it starts as a Follower.
#[derive(Debug)]
pub struct RaftNode {
  id: u16,
  state: RaftState,
  current_leader: Option<u16>,
  cluster_info: HashMap<u16, SocketAddr>,
  time_for_election: Option<Instant>,
}

impl RaftNode {
  pub fn new(id: u16, server_info: HashMap<u16, SocketAddr>) -> Self {
    let mut new_node = Self {
      id,
      state: RaftState::new(),
      current_leader: None,
      cluster_info: server_info,
      time_for_election: None,
    };
    new_node.wait_to_another_election();
    new_node
  }

  /// Is in charge of broadcasting the heartbeats to all others the nodes, only if
  /// the node is the leader.
  pub fn broadcast_heartbeat(&mut self) {
    if self.state.is_leader() {
      let id = self.id;
      info!("Broadcasting heartbeats from {id}");
    }
  }

  /// Checks if it is time for a new election and start it. Otherwise it doesn't
  /// do anything.
  pub async fn handle_timeout(&mut self) {
    if self.is_election_time() {
      let id = self.id;
      let term = self.state.term();
      info!("Starting an election from node {id} in term {term}");
      self.start_election().await;
    }
  }

  async fn start_election(&mut self) {
    self.state.prepare_for_election();
    let total_votes = self.cluster_info.len() as i32;
    let positive_votes = 0;
    for (id, address) in &self.cluster_info {
      info!("Request vote for {id}");
      match self.request_for_vote(&address).await {
        Ok(vote_response) => {
          info!("Vote requested");
        },
        Err(e) => {
          error!("Cannot emmit the vote {e}");
        },
      }
    }
    let id = self.id;
    match positive_votes > total_votes / 2 {
      true => {
        info!("Node {id} won the election");
      },
      false => {
        info!("Node {id} lost the election");
        self.wait_to_another_election();
        self.state.election_lost();
      },
    }
  }

  async fn request_for_vote(
    &self,
    address: &SocketAddr,
  ) -> Result<RaftVoteResponse, String> {
    let vote = RaftRequestVote::new(self.state.term(), self.id_node());
    let client = reqwest::Client::new();
    let base_url = format!("http://{}", address);
    let response = client
      .post(base_url)
      .json(&vote)
      .send()
      .await
      .map_err(|e| e.to_string())?;
    response
      .json::<RaftVoteResponse>()
      .await
      .map_err(|e| e.to_string())
  }

  /// Sets the new instante for an election.
  /// The paper suggests a random timeout to avoid ties in the election process.
  fn wait_to_another_election(&mut self) {
    let mut rng = rand::thread_rng();
    // let millis = Duration::from_millis(rng.gen_range(150..300));
    let millis = Duration::from_millis(rng.gen_range(4000..5000));
    self.time_for_election = Some(Instant::now() + millis);
  }

  /// Checks if it's time for a new election.
  fn is_election_time(&self) -> bool {
    match self.time_for_election {
      Some(tfe) => Instant::now() > tfe && self.state.is_follower(),
      None => false,
    }
  }

  fn id_node(&self) -> u16 {
    self.id
  }

  fn is_follower(&self) -> bool {
    self.state.is_follower()
  }

  /// Handle the reception of an entry from another node.
  /// For now I will only implement the election process.
  pub fn process(&mut self, entry: RaftEntry) -> RaftEntryResponse {
    if entry.term() < self.state.term() {
      RaftEntryResponse::failure(self.state.term());
    }
    self.wait_to_another_election();
    RaftEntryResponse::success(self.state.term())
  }
}

/// The result of an election
#[derive(Clone, Debug, Eq, PartialEq)]
enum ElectionResult {
  Win { node: u16 },
  Lose,
  Tie,
}

#[cfg(test)]
mod tests {

  use super::*;
  use std::{str::FromStr, thread};
  static ELECTION_TIMEOUT: u64 = 6;

  #[test]
  fn create_new_node() {
    let mut server_addres = HashMap::new();
    server_addres.insert(
      1u16,
      SocketAddr::from_str("127.0.0.1:7001").unwrap(),
    );
    server_addres.insert(
      2u16,
      SocketAddr::from_str("127.0.0.1:7002").unwrap(),
    );

    let node = RaftNode::new(1u16, server_addres);
    assert_eq!(node.id_node(), 1u16);
    assert_eq!(node.is_follower(), true);
    assert_eq!(node.is_election_time(), false);
  }

  #[test]
  fn new_node_with_time_elapsed() {
    let mut server_addres = HashMap::new();
    server_addres.insert(
      1u16,
      SocketAddr::from_str("127.0.0.1:7001").unwrap(),
    );
    server_addres.insert(
      2u16,
      SocketAddr::from_str("127.0.0.1:7002").unwrap(),
    );

    let mut node = RaftNode::new(1u16, server_addres);
    node.wait_to_another_election();
    thread::sleep(Duration::from_secs(ELECTION_TIMEOUT));

    assert_eq!(node.id_node(), 1u16);
    assert_eq!(node.is_follower(), true);
    assert_eq!(node.is_election_time(), true);
  }

  #[test]
  fn process_with_time_elapsed() {
    let mut server_addres = HashMap::new();
    server_addres.insert(
      1u16,
      SocketAddr::from_str("127.0.0.1:7001").unwrap(),
    );
    server_addres.insert(
      2u16,
      SocketAddr::from_str("127.0.0.1:7002").unwrap(),
    );

    let mut node = RaftNode::new(1u16, server_addres);
    let entry = RaftEntry::new_heartbeat(2, 2, 0, 0, 0);
    assert_eq!(node.is_election_time(), false);
    let response = node.process(entry);
    thread::sleep(Duration::from_secs(ELECTION_TIMEOUT));

    assert_eq!(response.entry_term(), 0);
    assert_eq!(response.entry_success(), true);
    assert_eq!(node.is_election_time(), true);
  }

  #[test]
  fn process_with_term_greater_than() {
    let mut server_addres = HashMap::new();
    server_addres.insert(
      1u16,
      SocketAddr::from_str("127.0.0.1:7001").unwrap(),
    );
    server_addres.insert(
      2u16,
      SocketAddr::from_str("127.0.0.1:7002").unwrap(),
    );

    let mut node = RaftNode::new(1u16, server_addres);
    let entry = RaftEntry::new_heartbeat(2, 2, 0, 0, 0);
    assert_eq!(node.is_election_time(), false);
    let response = node.process(entry);

    assert_eq!(response.entry_term(), 0);
    assert_eq!(response.entry_success(), true);
  }

  #[test]
  fn process_with_term_equal() {
    let mut server_addres = HashMap::new();
    server_addres.insert(
      1u16,
      SocketAddr::from_str("127.0.0.1:7001").unwrap(),
    );
    server_addres.insert(
      2u16,
      SocketAddr::from_str("127.0.0.1:7002").unwrap(),
    );

    let mut node = RaftNode::new(1u16, server_addres);
    let entry = RaftEntry::new_heartbeat(0, 2, 0, 0, 0);
    let response = node.process(entry);

    assert_eq!(response.entry_term(), 0);
    assert_eq!(response.entry_success(), true);
  }

  #[test]
  fn process_with_term_less_than() {
    let mut server_addres = HashMap::new();
    server_addres.insert(
      1u16,
      SocketAddr::from_str("127.0.0.1:7001").unwrap(),
    );
    server_addres.insert(
      2u16,
      SocketAddr::from_str("127.0.0.1:7002").unwrap(),
    );

    let mut node = RaftNode::new(1u16, server_addres);
    let entry = RaftEntry::new_heartbeat(4, 2, 0, 0, 0);
    let response = node.process(entry);

    // cannot be done yet
    // assert_eq!(response.entry_term(), 5);
    // assert_eq!(response.entry_success(), false);
  }
}
