# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Proof-of-concept implementation of the Raft consensus algorithm in Rust, based on the original paper "In Search of an Understandable Consensus Algorithm". Currently focused on leader election; log replication is not yet implemented. Uses HTTP/JSON (axum) instead of gRPC for inter-node communication.

## Build & Test Commands

```bash
cargo build              # Build the project
cargo test               # Run all tests
cargo test <test_name>   # Run a single test (e.g., cargo test create_new_node)
cargo run                # Run a node (requires .env file, see below)
```

No linter or formatter is configured. Use `cargo fmt` and `cargo clippy` as needed.

## Running a Cluster

Each node requires a `.env` file with: `HOST`, `ID_NODE`, `ID_NODES` (comma-separated peer IDs), `HOST_<N>` for each peer, `RUST_LOG`, and `NODE_CLIENT_TIMEOUT_MS`. Minimum 3 nodes needed. See README.md for example configs.

Quick setup with zellij:
```bash
mkdir -p tmp/cluster_{1,2,3}
# Place a .env in each tmp/cluster_N/ directory
zellij -l code-layout.kdl
```

## Architecture

**Layered structure with three modules:**

- **`raft`** — `RaftServer` orchestrates startup: creates the node, wires up HTTP routes, and spawns a background tokio task that loops calling `handle_timeout()` (triggers elections) and `broadcast_heartbeat()` on the node. Configuration (`RaftServerConfig`) reads from env vars.

- **`node`** — Core Raft logic, no framework dependencies:
  - `RaftNode` — Owns the state machine, cluster membership map, election timer (randomized 150-300ms), and a `Box<dyn NodeClient>` for inter-node RPCs. Exposes `ack_heartbeat`, `answer_vote`, `handle_command`.
  - `RaftState` — Tracks term, position (Follower/Candidate/Leader), vote, and log. Implements vote-granting rules and heartbeat processing.
  - `entry` — Data types for the Raft protocol: `RaftEntry` (Heartbeat/LogEntry variants), `RaftRequestVote`, and their response types.
  - `node_client` — `NodeClient` trait (async, object-safe) and `HttpNodeClient` implementation using reqwest. The trait enables mock clients for testing.

- **`api`** — Axum HTTP handlers. All endpoints share `Arc<Mutex<RaftNode>>` state:
  - `POST /heartbeat` — Receives AppendEntries (heartbeats)
  - `POST /vote` — Receives RequestVote RPCs
  - `GET /info` — Returns node id, term, leader, last vote
  - `POST /command` — Submits a command; redirects to leader if not leader

**`error`** — Centralized error types mapping to HTTP status codes.

## Key Design Decisions

- Node state is behind `tokio::sync::Mutex` (not `std::Mutex`) since locks are held across `.await` points.
- `NodeClient` is a trait object (`Box<dyn NodeClient>`) to allow swapping implementations (HTTP vs mock).
- Election timeout uses `std::time::Instant`, not tokio timers — the background loop polls continuously.
