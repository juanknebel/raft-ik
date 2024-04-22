use dotenv;
use env_logger;

use crate::raft::server::{RaftServerConfig, RaftServerRpc};

mod api;
mod client;
mod error;
mod node;
mod raft;

#[tokio::main]
async fn main() {
  dotenv::dotenv().ok();
  env_logger::builder().format_timestamp_millis().init();

  // -- Initialize the configuration -- //
  let config = RaftServerConfig::new();

  // -- Creates new Raft Server -- //
  //  let mut server = RaftServerRest::new(config);

  let server = RaftServerRpc::new(config);

  // -- Start Raft Server -- //
  server.start().await;
}
