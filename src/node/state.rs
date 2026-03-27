use super::entry::{Entry, EntryResult, Vote, VoteResult};

#[derive(Debug)]
pub struct RaftState {
  current_term: u64,
  current_position: RaftPosition,
  vote_for: Option<u16>,
  log: Vec<Entry>,
  commit_index: u64,
  last_applied: u64,
}

impl RaftState {
  pub fn new() -> Self {
    Self {
      current_term: 0,
      current_position: RaftPosition::Follower,
      vote_for: None,
      log: Vec::new(),
      commit_index: 0,
      last_applied: 0,
    }
  }

  /// In order to prepare to the election it needs to increment the actual term,
  /// transition to a Candidate state and vote for itself.
  pub fn prepare_for_election(&mut self, node_id: u16) {
    self.current_term = self.current_term + 1;
    self.current_position = RaftPosition::Candidate;
    self.vote_for = Some(node_id)
  }

  pub fn last_vote(&self) -> Option<u16> {
    self.vote_for
  }

  pub fn term(&self) -> u64 {
    self.current_term
  }

  fn logs(&self) -> &Vec<Entry> {
    self.log.as_ref()
  }

  /// Process an entry send by another node.
  /// At this point I assume is only a heartbeat.
  pub fn process(&mut self, an_entry: &Entry) -> EntryResult {
    if an_entry.is_heartbeat() {
      if an_entry.term() < self.term() {
        return EntryResult::failure(self.term());
      }
      self.current_position = RaftPosition::Follower;
      self.vote_for = None;
      self.current_term = an_entry.term();
      EntryResult::success(self.term())
    } else {
      EntryResult::failure(self.term())
    }
  }

  /// Decides if this state could emmit a positive or negative vote.
  /// If the vote is too old it is a negative vote.
  /// If the state doesn´t vote for anyone yet, then is positive vote.
  /// If the log of the candidate is up to date with the state, then the vote is
  /// positive. If the log of the candidate is old, then the vote is negative.
  pub fn answer_vote(&mut self, a_vote: &Vote) -> VoteResult {
    if self.term() > a_vote.term() {
      return VoteResult::failure(self.term());
    }
    return match self.vote_for {
      Some(_) => VoteResult::failure(self.term()),
      None => {
        if self.log_is_up_to_date(&a_vote) {
          self.vote_for = Some(a_vote.candidate());
          VoteResult::success(self.term())
        } else {
          VoteResult::failure(self.term())
        }
      },
    };
  }

  /// How to know if the log is update (from the state perspective):
  /// TODO: until the log replication is developed this method will return
  /// always true.
  /// * the log is empty
  /// * the term in the last entry is lower than the request for vote
  /// * the log terms are equal but the length of the logs is lower than the
  ///   request for vote.
  fn log_is_up_to_date(&self, a_vote: &Vote) -> bool {
    match self.log.last() {
      Some(last_entry) => {
        let size = self.log.len() as u64;
        if last_entry.term() == a_vote.log_term() {
          size < a_vote.log_index()
        } else {
          last_entry.term() < a_vote.log_term()
        }
      },
      None => true,
    }
  }

  /// Resolve the election based on the votes.
  /// If the positive votes are more than 50% then transition the state to
  /// leader, otherwise revert to follower and updates for which node it voted.
  pub fn resolve_election(
    &mut self,
    positive_votes: i32,
    total_votes: i32,
  ) -> bool {
    return if positive_votes > total_votes / 2 {
      self.current_position = RaftPosition::Leader;
      true
    } else {
      self.current_position = RaftPosition::Follower;
      self.vote_for = None;
      false
    };
  }

  pub fn position(&self) -> RaftPosition {
    self.current_position.clone()
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
    assert_eq!(state.position(), RaftPosition::Follower);
    assert_eq!(state.term(), 0);
    assert_eq!(state.logs().is_empty(), true);
  }

  #[test]
  fn prepare_for_election() {
    let mut state = RaftState::new();
    state.prepare_for_election(1u16);

    assert_eq!(state.position(), RaftPosition::Candidate);
    assert_eq!(state.term(), 1);
    assert_eq!(state.logs().is_empty(), true);
    assert_eq!(state.last_vote(), Some(1u16))
  }

  #[test]
  fn log_up_to_date() {
    let a_vote = Vote::new(5u64, 2u16);
    let a_state = RaftState::new();
    let up_to_date = a_state.log_is_up_to_date(&a_vote);
    assert_eq!(up_to_date, true);
  }

  #[test]
  fn answer_vote_ok_not_voted() {
    let a_vote = Vote::new(1u64, 2u16);
    let mut a_state = RaftState::new();
    let response_vote = a_state.answer_vote(&a_vote);

    assert_eq!(response_vote.vote_was_granted(), true);
    assert_eq!(response_vote.vote_term(), 0u64);
  }

  #[test]
  fn answer_vote_not_ok_voted() {
    let a_vote = Vote::new(1u64, 2u16);
    let mut a_state = RaftState::new();
    a_state.prepare_for_election(3u16);
    let response_vote = a_state.answer_vote(&a_vote);

    assert_eq!(response_vote.vote_was_granted(), false);
    assert_eq!(response_vote.vote_term(), 1u64);
  }

  #[test]
  fn answer_vote_not_ok_term() {
    let a_vote = Vote::new(1u64, 2u16);
    let mut a_state = RaftState::new();
    a_state.prepare_for_election(1u16);
    a_state.prepare_for_election(1u16);
    let response_vote = a_state.answer_vote(&a_vote);

    assert_eq!(response_vote.vote_was_granted(), false);
    assert_eq!(response_vote.vote_term(), 2u64);
  }

  /// Until the log replication this test is useless
  #[test]
  fn answer_vote_not_ok_log_not_updated() {
    let a_vote = Vote::new(2u64, 2u16);
    let mut a_state = RaftState::new();
    let response_vote = a_state.answer_vote(&a_vote);

    assert_eq!(response_vote.vote_was_granted(), true); // this should be false
    assert_eq!(response_vote.vote_term(), 0);
  }

  #[test]
  fn process_heartbeat_fail() {
    let an_entry = Entry::new_heartbeat(1u64, 1u16, 0, 0, 0);
    let mut a_state = RaftState::new();
    a_state.current_term = 2u64;
    let response_heartbeat = a_state.process(&an_entry);

    assert_eq!(response_heartbeat.entry_success(), false);
    assert_eq!(response_heartbeat.entry_term(), 2u64);
  }

  #[test]
  fn process_heartbeat_success() {
    let an_entry = Entry::new_heartbeat(1u64, 1u16, 0, 0, 0);
    let mut a_state = RaftState::new();
    let response_heartbeat = a_state.process(&an_entry);

    assert_eq!(response_heartbeat.entry_success(), true);
    assert_eq!(response_heartbeat.entry_term(), 1u64);
    assert_eq!(a_state.current_term, 1u64);
  }
}
