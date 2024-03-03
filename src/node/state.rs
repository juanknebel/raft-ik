use super::entry::RaftEntry;

#[derive(Debug)]
pub struct RaftState {
  current_term: u64,
  current_position: RaftPosition,
  vote_for: Option<u16>,
  log: Vec<RaftEntry>,
}

impl RaftState {
  pub fn new() -> Self {
    Self {
      current_term: 0,
      current_position: RaftPosition::Follower,
      vote_for: None,
      log: Vec::new(),
    }
  }

  pub fn prepare_for_election(&mut self) {
    self.current_term = self.current_term + 1;
    self.current_position = RaftPosition::Candidate
  }

  pub fn election_lost(&mut self) {
    self.current_position = RaftPosition::Follower;
  }

  pub fn is_follower(&self) -> bool {
    self.current_position == RaftPosition::Follower
  }

  pub fn is_leader(&self) -> bool {
    self.current_position == RaftPosition::Leader
  }

  fn last_vote(&self) -> Option<u16> {
    self.vote_for
  }

  pub fn term(&self) -> u64 {
    self.current_term
  }

  fn logs(&self) -> &Vec<RaftEntry> {
    self.log.as_ref()
  }
}

/// Identifies the position that the node has it.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum RaftPosition {
  Leader,
  Follower,
  Candidate,
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn create_new_state() {
    let state = RaftState::new();

    assert_eq!(state.last_vote(), None);
    assert_eq!(state.is_follower(), true);
    assert_eq!(state.term(), 0);
    assert_eq!(state.logs().is_empty(), true);
  }
}
