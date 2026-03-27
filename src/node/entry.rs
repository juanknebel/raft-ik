use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Entry {
  Heartbeat {
    message: Message,
  },
  LogEntry {
    message: Message,
    commands: Vec<String>,
  },
}

impl Entry {
  pub fn term(&self) -> u64 {
    match self {
      Self::Heartbeat {
        message,
      } => message.term,
      Self::LogEntry {
        message, ..
      } => message.term,
    }
  }

  pub fn leader_id(&self) -> u16 {
    match self {
      Self::Heartbeat {
        message,
      } => message.leader_id,
      Self::LogEntry {
        message, ..
      } => message.leader_id,
    }
  }

  pub fn is_heartbeat(&self) -> bool {
    match self {
      Self::Heartbeat {
        ..
      } => true,
      Self::LogEntry {
        ..
      } => false,
    }
  }

  pub fn new_heartbeat(
    term: u64,
    leader_id: u16,
    prev_log_index: u64,
    prev_log_term: u64,
    leader_commit: u64,
  ) -> Self {
    let message = Message {
      term,
      leader_id,
      prev_log_index,
      prev_log_term,
      leader_commit,
    };
    Entry::Heartbeat {
      message,
    }
  }

  fn new_log_entry(
    term: u64,
    leader_id: u16,
    prev_log_index: u64,
    prev_log_term: u64,
    leader_commit: u64,
    commands: Vec<String>,
  ) -> Self {
    let message = Message {
      term,
      leader_id,
      prev_log_index,
      prev_log_term,
      leader_commit,
    };
    Entry::LogEntry {
      message,
      commands,
    }
  }
}

impl core::fmt::Display for Entry {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self:?}")
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
  pub term: u64,
  pub leader_id: u16,
  pub prev_log_index: u64,
  pub prev_log_term: u64,
  pub leader_commit: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryResult {
  term: u64,
  success: bool,
}

impl EntryResult {
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Vote {
  term: u64,
  candidate_id: u16,
  last_log_index: u64,
  last_log_term: u64,
}

impl Vote {
  pub fn new(term: u64, candidate_id: u16) -> Self {
    Vote {
      term,
      candidate_id,
      last_log_index: 0,
      last_log_term: 0,
    }
  }

  pub fn term(&self) -> u64 {
    self.term
  }

  pub fn candidate(&self) -> u16 {
    self.candidate_id
  }

  pub fn log_index(&self) -> u64 {
    self.last_log_index
  }

  pub fn log_term(&self) -> u64 {
    self.last_log_term
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VoteResult {
  term: u64,
  vote_granted: bool,
}

impl VoteResult {
  pub fn failure(term: u64) -> Self {
    Self {
      term,
      vote_granted: false,
    }
  }

  pub fn success(term: u64) -> Self {
    Self {
      term,
      vote_granted: true,
    }
  }

  pub fn vote_was_granted(&self) -> bool {
    self.vote_granted
  }

  pub fn vote_term(&self) -> u64 {
    self.term
  }
}
