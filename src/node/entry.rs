use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub enum RaftEntry {
  Heartbeat {
    message: RaftMessage,
  },
  LogEntry {
    message: RaftMessage,
    commands: Vec<String>,
  },
}

impl RaftEntry {
  pub fn term(&self) -> u64 {
    match self {
      Self::Heartbeat { message } => message.term,
      Self::LogEntry { message, .. } => message.term,
    }
  }

  pub fn is_hearbet(&self) -> bool {
    match self {
      Self::Heartbeat { .. } => true,
      Self::LogEntry { .. } => false,
    }
  }

  pub fn new_heartbeat(
    term: u64,
    leader_id: u16,
    prev_log_index: u64,
    prev_log_term: u64,
    leader_commit: u64,
  ) -> Self {
    let message = RaftMessage {
      term,
      leader_id,
      prev_log_index,
      prev_log_term,
      leader_commit,
    };
    RaftEntry::Heartbeat { message }
  }

  pub fn new_log_entry(
    term: u64,
    leader_id: u16,
    prev_log_index: u64,
    prev_log_term: u64,
    leader_commit: u64,
    commands: Vec<String>,
  ) -> Self {
    let message = RaftMessage {
      term,
      leader_id,
      prev_log_index,
      prev_log_term,
      leader_commit,
    };
    RaftEntry::LogEntry { message, commands }
  }
}

impl core::fmt::Display for RaftEntry {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self:?}")
  }
}

#[derive(Debug, Deserialize)]
struct RaftMessage {
  pub term: u64,
  pub leader_id: u16,
  pub prev_log_index: u64,
  pub prev_log_term: u64,
  pub leader_commit: u64,
}

#[derive(Debug, Serialize)]
pub struct RaftEntryResponse {
  term: u64,
  success: bool,
}

impl RaftEntryResponse {
  pub fn success(term: u64) -> Self {
    Self {
      term,
      success: true,
    }
  }

  pub fn failure(term: u64) -> Self {
    Self {
      term,
      success: false,
    }
  }

  pub fn entry_term(&self) -> u64 {
    self.term
  }

  pub fn entry_success(&self) -> bool {
    self.success
  }
}
