# High Availability & Distributed Consensus Expert

Expert analysis of the Raft consensus protocol implementation, comparing against the original paper "In Search of an Understandable Consensus Algorithm". Covers protocol correctness, concurrency issues, partition tolerance, and missing safety guarantees.

---

## 1. CRITICAL ISSUES

### 1.1 Busy-loop without sleep in `background_tasks`

**File:** `src/raft/server.rs`, lines 120-129

```rust
async fn background_tasks(node: Arc<Mutex<RaftNode>>) {
  loop {
    let mut node_lock = node.lock().await;
    node_lock.handle_timeout().await;
    drop(node_lock);

    let mut node_lock = node.lock().await;
    node_lock.broadcast_heartbeat().await;
    drop(node_lock);
  }
}
```

The `loop` has no `sleep` or `yield`. This causes:

1. **Mutex starvation:** The loop continuously acquires and releases the lock without pause, preventing HTTP handlers from ever acquiring it. The node cannot respond to requests from other nodes.
2. **100% CPU usage:** A pure spin-loop that consumes an entire core.
3. **Functional deadlock during elections:** `start_election()` makes HTTP calls while holding the lock. If node A requests a vote from node B, node B cannot respond because its own spin-loop holds the lock.
4. **Same issue with heartbeats:** `broadcast_heartbeat()` makes HTTP calls while holding the lock.

**Solution:** Add `tokio::time::sleep` to the loop, and perform network calls outside the lock scope.

### 1.2 Vote count does not include self-vote

**File:** `src/node/node.rs`, lines 84-113

```rust
async fn start_election(&mut self) {
    self.state.prepare_for_election(self.id_node()); // votes for itself
    let total_votes = self.cluster_info.len() as i32;  // only counts peers
    let mut positive_votes = 0;  // does NOT include self-vote
```

Per the paper (section 5.2): "A candidate first votes for itself." But `positive_votes` starts at 0 and `total_votes` only counts peers.

- 3-node cluster: requires 2 peer votes when it should only need 1 (self-vote + 1 peer).
- 5-node cluster: requires 3 peer votes when it should only need 2.

**Solution:**
```rust
let total_votes = self.cluster_info.len() as i32 + 1; // +1 for self
let mut positive_votes = 1; // self-vote
```

### 1.3 Sequential election blocks the node

**File:** `src/node/node.rs`, lines 89-103

```rust
for (id, address) in &self.cluster_info {
    match self.client.request_for_vote(&address, &a_vote).await {
```

Votes are requested sequentially, one by one. If a node is down, the candidate blocks waiting for the timeout before moving to the next one. During all that time, the mutex is locked. Per the paper, vote requests should be sent in parallel.

### 1.4 `answer_vote` does not update term when receiving a higher term

**File:** `src/node/state.rs`, lines 66-81

```rust
pub fn answer_vote(&mut self, a_vote: &RaftRequestVote) -> RaftVoteResponse {
    if self.term() > a_vote.term() {
      return RaftVoteResponse::failure(self.term());
    }
    return match self.vote_for {
      Some(_) => RaftVoteResponse::failure(self.term()),
      None => { ... },
    };
}
```

Per the paper (section 5.1): **"If RPC request or response contains term T > currentTerm: set currentTerm = T, convert to follower."** The code does not do this. A node that already voted in term 5 rejects a vote for term 7, when it should update its term to 7, reset `vote_for` to `None`, and grant the vote. This can cause two leaders in different terms.

### 1.5 Heartbeat responses ignored (possible split-brain)

**File:** `src/node/node.rs`, lines 56-68

```rust
match self.client.send_heartbeat(&address, &heartbeat).await {
    Ok(_) => {},  // <-- completely ignores the response
    Err(e) => { error!(...) },
}
```

The leader ignores the heartbeat response. Per the paper: "If RPC response contains term T > currentTerm: set currentTerm = T, convert to follower." If a follower responds with a higher term, the current leader should step down. By ignoring the response, **two simultaneous leaders** (split-brain) are possible.

---

## 2. MAJOR ISSUES

### 2.1 `RaftRequestVote` always sends `last_log_index: 0, last_log_term: 0`

**File:** `src/node/entry.rs`, lines 143-149

```rust
pub fn new(term: u64, candidate_id: u16) -> Self {
    RaftRequestVote { term, candidate_id, last_log_index: 0, last_log_term: 0 }
}
```

The candidate never communicates its actual log information. This violates the safety restriction (section 5.4.1, "Election restriction"): a candidate with an empty log could win over nodes with committed entries, causing data loss.

### 2.2 Heartbeat accepts any term >= without log validation

**File:** `src/node/state.rs`, lines 47-59

The receiver must verify that its log contains an entry at `prevLogIndex` whose term matches `prevLogTerm`. The parameters are passed but never verified.

### 2.3 No state persistence

Per the paper (Figure 2), `currentTerm`, `votedFor`, and `log[]` must be persisted to disk before responding to any RPC. Without persistence, a restarted node loses its `vote_for` and can vote for a second candidate in the same term.

### 2.4 Node can send messages to itself

**File:** `src/node/node.rs`, lines 59 and 89

`cluster_info` is built from `ID_NODES` in `.env`. There is no filter to exclude the node itself. If its own ID is included, the node sends heartbeats and votes to itself, causing a real deadlock (waits for an HTTP response from itself while holding the lock).

### 2.5 `handle_command` does nothing

**File:** `src/node/node.rs`, lines 181-198

```rust
RaftPosition::Leader => {
    info!("Processing the command: {}", a_command);
    Ok(())
},
```

The leader logs the command and returns Ok(). It is not added to the log or replicated. The `/command` endpoint is a no-op.

### 2.6 `unwrap()` in command handler

**File:** `src/api/api.rs`, line 90

```rust
leader: node_lock.leader().unwrap(),
```

If `current_leader` is `None`, this causes a server panic.

---

## 3. MINOR ISSUES

### 3.1 Typo: `is_hearbaet`
**File:** `src/node/entry.rs`, line 37. Should be `is_heartbeat`.

### 3.2 10ms HTTP timeout
**File:** `src/raft/server.rs`, line 49. Too aggressive for HTTP with JSON. Causes constant failures and unstable elections.

### 3.3 No interval between leader heartbeats
The leader sends heartbeats as fast as possible (spin-loop). Should have a reasonable interval (e.g., 50ms).

### 3.4 Empty `handle_timeout` test
**File:** `src/node/node.rs`, line 291.

### 3.5 `new_log_entry` defined but never used
**File:** `src/node/entry.rs`, line 67.

---

## 4. CONCURRENCY ANALYSIS

The architecture uses a single `Arc<Mutex<RaftNode>>`. Implications:

1. **Total serialization:** Every operation competes for a single mutex.
2. **Network operations under lock:** The most dangerous anti-pattern in lock-based concurrency.
3. **Functional deadlock:** The node blocks waiting for a network response from another node that is in turn waiting to acquire its own lock.

**Recommended solution:** Actor pattern with `tokio::mpsc` — a single task owns the `RaftNode` and receives messages via channel. Network operations outside the lock.

---

## 5. MISSING FEATURES

| Feature | Status | Priority |
|---|---|---|
| Log replication (full AppendEntries) | Not implemented | Critical |
| State persistence (term, votedFor, log) | Not implemented | Critical |
| Applying entries to state machine | Not implemented | Critical |
| Snapshots / Log compaction | Not implemented | High |
| Membership changes (AddServer/RemoveServer) | Not implemented | Medium |
| Linearizable reads | Not implemented | Medium |
| Pre-vote protocol (extension) | Not implemented | Low |

---

## 6. SUMMARY BY SEVERITY

**CRITICAL:**
1. Spin-loop without sleep — starvation and 100% CPU
2. Network calls under lock — functional deadlock
3. Vote count does not include self-vote — incorrect elections
4. `answer_vote` does not update term on higher term — safety violation
5. Heartbeat responses ignored — possible split-brain

**MAJOR:**
1. `RaftRequestVote` always sends `last_log_index=0`
2. No critical state persistence
3. Node can send requests to itself (deadlock)
4. Commands are not stored or replicated
5. `unwrap()` in production

**MINOR:**
1. Typo `is_hearbaet`
2. 10ms HTTP timeout
3. No heartbeat interval
4. Empty test and dead code
