mod api;
mod error;
mod node;
mod raft;
use std::{net::SocketAddr, str::FromStr};

use dotenv;
use env_logger;
use log::info;

use crate::raft::server::{RaftServer, RaftServerConfig};

#[tokio::main]
async fn main() {
  dotenv::dotenv().ok();
  env_logger::init();

  // -- Initialize the configuration -- //
  let config = RaftServerConfig::new();

  // -- Creates new Raft Server -- //
  let mut server = RaftServer::new(&config);

  // -- Start Raft Server -- //
  server.start().await;

  // -- Start the the web server -- //
  let addr = SocketAddr::from_str(&config.host()).unwrap();
  info!("[Listening on {addr}]");
  axum::Server::bind(&addr)
    .serve(server.routes().into_make_service())
    .await
    .unwrap();
}
