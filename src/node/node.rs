use std::{
  collections::HashMap,
  net::SocketAddr,
  time::{Duration, Instant},
};

use log::{error, info};
use rand::Rng;

use crate::node::{
  entry::RaftVoteResponse, node_client::NodeClient, state::RaftPosition,
};

use super::{
  entry::{RaftEntry, RaftEntryResponse, RaftRequestVote},
  state::RaftState,
};

/// The RaftNode contains all the necessary information to perform all the
/// actions to mantain a consensus between the whole network of nodes in a
/// distributed system. The node at any given moment could be only one the
/// following positions: Follower, Candidate, Leader. Every time the node came
/// to life it starts as a Follower.
#[derive(Debug)]
pub struct RaftNode {
  id: u16,
  state: RaftState,
  current_leader: Option<u16>,
  cluster_info: HashMap<u16, SocketAddr>,
  time_for_election: Option<Instant>,
  client: Box<dyn NodeClient>,
}

impl RaftNode {
  pub fn new(
    id: u16,
    server_info: HashMap<u16, SocketAddr>,
    node_client: impl NodeClient,
  ) -> Self {
    let node_client = Box::new(node_client);
    let mut new_node = Self {
      id,
      state: RaftState::new(),
      current_leader: None,
      cluster_info: server_info,
      time_for_election: None,
      client: node_client,
    };
    new_node.wait_to_another_election();
    new_node
  }

  /// Is in charge of broadcasting the heartbeats to all others the nodes, only
  /// if the node is the leader.
  pub async fn broadcast_heartbeat(&mut self) {
    if self.state.position() == RaftPosition::Leader {
      let heartbeat =
        RaftEntry::new_heartbeat(self.state.term(), self.id, 0, 0, 0);
      for (id, address) in &self.cluster_info {
        //        match send_heartbeat(&address, &heartbeat).await {
        match self.client.send_heartbeat(&address, &heartbeat).await {
          Ok(_) => {},
          Err(e) => {
            error!("Cannot broadcast the heartbeat to cluster {id}. {e}");
          },
        }
      }
    }
  }

  /// Checks if it is time for a new election and start it. Otherwise, it
  /// doesn't do anything.
  pub async fn handle_timeout(&mut self) {
    if self.is_election_time() {
      info!(
        "Starting an election from node {} in term {}",
        self.id,
        self.state.term()
      );
      self.start_election().await;
    }
  }

  async fn start_election(&mut self) {
    self.state.prepare_for_election(self.id_node());
    let total_votes = self.cluster_info.len() as i32;
    let mut positive_votes = 0;
    let a_vote = RaftRequestVote::new(self.state.term(), self.id_node());
    for (id, address) in &self.cluster_info {
      info!("Sending a vote to node: {}", id);
      //      match request_for_vote(&address, &a_vote).await {
      match self.client.request_for_vote(&address, &a_vote).await {
        Ok(vote_response) => {
          if vote_response.vote_was_granted() {
            info!("A positive vote received from node {}", id);
            positive_votes = positive_votes + 1;
          }
        },
        Err(e) => {
          error!("Cannot emmit the vote for {id}. {e}");
        },
      }
    }

    if self.state.resolve_election(positive_votes, total_votes) {
      info!(
        "Node {} won the election for term {}",
        self.id,
        self.state.term()
      );
      self.current_leader = Some(self.id_node());
    }
    self.wait_to_another_election();
  }

  /// Sets the new instant for an election.
  /// The paper suggests a random timeout to avoid ties in the election process.
  fn wait_to_another_election(&mut self) {
    if self.state.position() != RaftPosition::Leader {
      let mut rng = rand::thread_rng();
      let millis = Duration::from_millis(rng.gen_range(150..300));
      //    let millis = Duration::from_secs(rng.gen_range(5..10));
      self.time_for_election = Some(Instant::now() + millis);
    }
  }

  /// Checks if it's time for a new election.
  fn is_election_time(&self) -> bool {
    match self.time_for_election {
      Some(tfe) => {
        Instant::now() > tfe && self.state.position() == RaftPosition::Follower
      },
      None => false,
    }
  }

  pub fn id_node(&self) -> u16 {
    self.id
  }

  pub fn current_term(&self) -> u64 {
    self.state.term()
  }

  pub fn leader(&self) -> Option<u16> {
    self.current_leader.clone()
  }

  pub fn vote_for(&self) -> Option<u16> {
    self.state.last_vote()
  }

  /// Handle the reception of a heartbeat entry from another node.
  pub fn ack_heartbeat(&mut self, entry: RaftEntry) -> RaftEntryResponse {
    let entry_response = self.state.process(&entry);
    if entry_response.entry_success() {
      self.current_leader = Some(entry.leader_id());
    }

    self.wait_to_another_election();
    entry_response
  }

  /// Handle the reception of request for vote.
  pub fn answer_vote(&mut self, a_vote: RaftRequestVote) -> RaftVoteResponse {
    let answer = self.state.answer_vote(&a_vote);
    info!(
      "Node {}, votes {} in request from {}",
      self.id,
      answer.vote_was_granted(),
      a_vote.candidate()
    );
    answer
  }

  /// Handles a new command into the cluster.
  /// If the node is the leader, then the command will be processed and
  /// replicated into the others nodes. (Not yet implemented).
  /// If the node isn't the leader, then raise an error with the address of the
  /// leader.
  pub fn handle_command(
    &mut self,
    a_command: impl Into<String> + std::fmt::Display,
  ) -> Result<(), Option<SocketAddr>> {
    match self.state.position() {
      RaftPosition::Leader => {
        info!("Processing the command: {}", a_command);
        Ok(())
      },
      _ => match self.current_leader {
        None => Err(None),
        Some(the_leader) => {
          let address_of_leader = self.cluster_info.get(&the_leader);
          Err(address_of_leader.cloned())
        },
      },
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::node::node_client::HttpNodeClient;
  use std::{str::FromStr, thread};

  static ELECTION_TIMEOUT: u64 = 6;

  #[test]
  fn create_new_node() {
    let node = new_default_raft_node();
    assert_eq!(node.id_node(), 1u16);
    assert_eq!(node.state.position(), RaftPosition::Follower);
    assert_eq!(node.is_election_time(), false);
  }

  fn new_default_raft_node() -> RaftNode {
    let mut server_addres = HashMap::new();
    server_addres.insert(
      1u16,
      SocketAddr::from_str("127.0.0.1:7001").unwrap(),
    );
    server_addres.insert(
      2u16,
      SocketAddr::from_str("127.0.0.1:7002").unwrap(),
    );

    let node_client = HttpNodeClient::new(Duration::from_millis(10u64));

    RaftNode::new(1u16, server_addres, node_client)
  }

  #[test]
  fn new_node_with_time_elapsed() {
    let mut node = new_default_raft_node();
    node.wait_to_another_election();
    thread::sleep(Duration::from_secs(ELECTION_TIMEOUT));

    assert_eq!(node.id_node(), 1u16);
    assert_eq!(node.state.position(), RaftPosition::Follower);
    assert_eq!(node.is_election_time(), true);
  }

  #[test]
  fn heartbeat_with_time_elapsed() {
    let mut node = new_default_raft_node();
    let entry = RaftEntry::new_heartbeat(2, 2, 0, 0, 0);
    assert_eq!(node.is_election_time(), false);
    let response = node.ack_heartbeat(entry);
    thread::sleep(Duration::from_secs(ELECTION_TIMEOUT));

    assert_eq!(response.entry_term(), 2);
    assert_eq!(response.entry_success(), true);
    assert_eq!(node.is_election_time(), true);
  }

  #[test]
  fn heartbeat_with_term_greater_than() {
    let mut node = new_default_raft_node();
    let entry = RaftEntry::new_heartbeat(2, 2, 0, 0, 0);
    assert_eq!(node.is_election_time(), false);
    let response = node.ack_heartbeat(entry);

    assert_eq!(response.entry_term(), 2);
    assert_eq!(response.entry_success(), true);
  }

  #[test]
  fn heartbeat_with_term_equal() {
    let mut node = new_default_raft_node();
    let entry = RaftEntry::new_heartbeat(0, 2, 0, 0, 0);
    let response = node.ack_heartbeat(entry);

    assert_eq!(response.entry_term(), 0);
    assert_eq!(response.entry_success(), true);
  }

  #[test]
  fn heartbeat_with_term_less_than() {
    let mut node = new_default_raft_node();
    let entry = RaftEntry::new_heartbeat(4, 2, 0, 0, 0);
    let _response = node.ack_heartbeat(entry);

    // cannot be done yet
    // assert_eq!(response.entry_term(), 5);
    // assert_eq!(response.entry_success(), false);
  }

  #[test]
  fn handle_timeout() {}
}
