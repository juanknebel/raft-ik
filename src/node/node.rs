use std::{
  collections::HashMap,
  net::SocketAddr,
  time::{Duration, Instant},
};

use log::info;
use rand::Rng;

use super::{
  entry::{RaftEntry, RaftEntryResponse},
  state::RaftState,
};

/// The RaftNode contains all the necessary information to perform all the actions to mantain
/// a consensus between the whole network of nodes in a distributed system.
/// The node at any given moment could be only one the following positions: Follower, Candidate,
/// Leader. Every time the node came to life it starts as a Follower.
#[derive(Debug)]
pub struct RaftNode {
  id: u16,
  position: RaftPosition,
  state: RaftState,
  current_leader: Option<u16>,
  cluster_info: RaftClusterInfo,
  time_for_election: Option<Instant>,
}

impl RaftNode {
  pub fn new(id: u16, server_info: HashMap<u16, SocketAddr>) -> Self {
    Self {
      id,
      position: RaftPosition::Follower,
      state: RaftState::new(),
      current_leader: None,
      cluster_info: RaftClusterInfo::new(server_info),
      time_for_election: None,
    }
  }

  /// Is in charge of broadcasting the heartbeats to all others the nodes, only if
  /// the node is the leader.
  pub fn broadcast_heartbeat(&mut self) {
    if self.position == RaftPosition::Leader {
      let id = self.id;
      info!("Broadcasting heartbeats from {id}");
    }
  }

  /// Checks if it is time for a new election and start it. Otherwise it doesn't
  /// do anything.
  pub fn handle_timeout(&mut self) {
    if self.is_election_time() {
      let id = self.id;
      info!("Starting an election from node {id}");
    }
  }

  /// Sets the new instante for an election.
  /// The paper suggests a random timeout to avoid ties in the election process.
  pub fn wait_to_another_election(&mut self) {
    let mut rng = rand::thread_rng();
    // let millis = Duration::from_millis(rng.gen_range(150..300));
    let millis = Duration::from_millis(rng.gen_range(4000..5000));
    self.time_for_election = Some(Instant::now() + millis);
  }

  /// Checks if it's time for a new election.
  pub fn is_election_time(&self) -> bool {
    match self.time_for_election {
      Some(tfe) => Instant::now() > tfe,
      None => false,
    }
  }

  pub fn id_node(&self) -> u16 {
    self.id
  }

  pub fn position_node(&self) -> RaftPosition {
    self.position.clone()
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

/// Identifies the position that the node has it.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum RaftPosition {
  Leader,
  Follower,
  Candidate,
}

/// Contains the addresses of the other nodes in the same cluster.
/// Every node has its own address, could be the same ip, but in a different port.
#[derive(Debug)]
pub struct RaftClusterInfo {
  server_address: HashMap<u16, SocketAddr>,
}

impl RaftClusterInfo {
  pub fn new(server_address: HashMap<u16, SocketAddr>) -> Self {
    Self { server_address }
  }

  pub fn get_address(&self, id: u16) -> Option<SocketAddr> {
    self.server_address.get(&id).cloned()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{str::FromStr, thread};
  static ELECTION_TIMEOUT: u64 = 6;

  #[test]
  fn create_new_cluster_info() {
    let mut server_addres = HashMap::new();
    server_addres.insert(
      1u16,
      SocketAddr::from_str("127.0.0.1:7001").unwrap(),
    );
    server_addres.insert(
      2u16,
      SocketAddr::from_str("127.0.0.1:7002").unwrap(),
    );

    let cluster_info = RaftClusterInfo::new(server_addres);
    assert_eq!(
      cluster_info.get_address(1u16).unwrap().to_string(),
      "127.0.0.1:7001"
    );
    assert_eq!(
      cluster_info.get_address(2u16).unwrap().to_string(),
      "127.0.0.1:7002"
    );
  }

  #[test]
  fn create_new_cluster_info_with_duplicate_value() {
    let mut server_addres = HashMap::new();
    server_addres.insert(
      1u16,
      SocketAddr::from_str("127.0.0.1:7001").unwrap(),
    );
    server_addres.insert(
      2u16,
      SocketAddr::from_str("127.0.0.1:7001").unwrap(),
    );

    let cluster_info = RaftClusterInfo::new(server_addres);
    assert_eq!(
      cluster_info.get_address(1u16).unwrap().to_string(),
      "127.0.0.1:7001"
    );
    assert_eq!(
      cluster_info.get_address(2u16).unwrap().to_string(),
      "127.0.0.1:7001"
    );
  }

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
    assert_eq!(node.position_node(), RaftPosition::Follower);
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
    assert_eq!(node.position_node(), RaftPosition::Follower);
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
