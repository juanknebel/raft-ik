use std::{
  collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc,
  time::Duration,
};

use axum::Router;
use log::info;
use tokio::sync::Mutex;

use crate::{
  api,
  node::{node::RaftNode, node_client::HttpNodeClient},
};

/// This a simple Server configuration to create the Node with its parameters.
#[derive(Clone)]
pub struct RaftServerConfig {
  server_identification: u16,
  peers_address: HashMap<u16, SocketAddr>,
  host_address: String,
  node_client_timeout: Duration,
}

impl RaftServerConfig {
  /// Creates the server configuration by reading the .env file located in the
  /// root folder. Only assumes a default value for the node client timeout.
  pub fn new() -> RaftServerConfig {
    dotenv::dotenv().ok();
    // -- Configuration --//
    let host = std::env::var("HOST").expect("Host undefined");
    let id_node: u16 = std::env::var("ID_NODE")
      .expect("NodeID undefined")
      .parse()
      .expect("Cannot parse the id node.");

    let nodes_string = std::env::var("ID_NODES").expect("NodeIDs undefined");
    let node_ids: Vec<&str> = nodes_string.split(',').collect();
    let mut address_info = HashMap::new();
    for a_node_id in node_ids {
      let id_number: u16 = a_node_id
        .parse()
        .expect("Cannot parse the node id {a_node_id}");
      let other_host = std::env::var("HOST_".to_string() + a_node_id)
        .expect("Host {a_node_id} undefined");
      let other_addr = SocketAddr::from_str(&other_host).unwrap();
      address_info.insert(id_number, other_addr);
    }
    let timeout_ms: u64 = std::env::var("NODE_CLIENT_TIMEOUT_MS")
      .unwrap_or("10".to_string())
      .parse()
      .expect("The node client timeout isn't a number");
    RaftServerConfig {
      server_identification: id_node,
      peers_address: address_info,
      host_address: host,
      node_client_timeout: Duration::from_millis(timeout_ms),
    }
  }

  pub fn host(&self) -> String {
    self.host_address.to_string()
  }

  fn addresses(&self) -> HashMap<u16, SocketAddr> {
    self.peers_address.clone()
  }

  fn server_id(&self) -> u16 {
    self.server_identification
  }

  fn node_timeout(&self) -> Duration {
    self.node_client_timeout.clone()
  }
}

/// The Raft Server that contains the Node wrapped in a sync Mutex and the
/// configuration needed to created it.
pub struct RaftServer {
  node: Arc<Mutex<RaftNode>>,
  router: Router,
}

impl RaftServer {
  pub fn new(config: &RaftServerConfig) -> RaftServer {
    // -- Node initialazing --//
    let node_client = HttpNodeClient::new(config.node_timeout());
    let node = RaftNode::new(
      config.server_id(),
      config.addresses(),
      node_client,
    );
    let arc_node = Arc::new(Mutex::new(node));

    // -- Routes --//
    let router = Router::new().merge(api::api::routes(Arc::clone(&arc_node)));
    RaftServer {
      node: Arc::clone(&arc_node),
      router,
    }
  }

  pub fn routes(&self) -> Router {
    self.router.clone()
  }

  /// Starts the Raft Server by spawning the task that need to be executed in
  /// the background.
  pub async fn start(&mut self) {
    info!("Starting the Raft Server");
    tokio::spawn(background_tasks(Arc::clone(&self.node)));
  }
}

/// This async function is in charge of executing the process of keeping the
/// entire system in a valid state.
/// * Indicates the node to handle the timeout, to update the new leader, to
/// keep informed who is the leader or to start a new election.
/// * Indicates the node to broadcast the heartbeat if necessary.
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
