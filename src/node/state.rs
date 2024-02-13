use super::entry::RaftEntry;

#[derive(Debug)]
pub struct RaftState {
  current_term: u64,
  vote_for: Option<u16>,
  log: Vec<RaftEntry>,
}

impl RaftState {
  pub fn new() -> Self {
    Self {
      current_term: 0,
      vote_for: None,
      log: Vec::new(),
    }
  }

  pub fn last_vote(&self) -> Option<u16> {
    self.vote_for
  }

  pub fn term(&self) -> u64 {
    self.current_term
  }

  pub fn logs(&self) -> &Vec<RaftEntry> {
    self.log.as_ref()
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn create_new_state() {
    let state = RaftState::new();

    assert_eq!(state.last_vote(), None);
    assert_eq!(state.term(), 0);
    assert_eq!(state.logs().is_empty(), true);
  }
}
