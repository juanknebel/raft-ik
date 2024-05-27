use std::{
  collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc,
  time::Duration,
};

use log::info;
use tokio::sync::Mutex;
use tonic::transport::Server;

use crate::{
  api::{
    api_proto::{
      raft_api,
      raft_api::{
        entry_point_server::EntryPointServer, health_server::HealthServer,
      },
      EntryPointService, HealthService,
    },
    core_api::{raft, raft::raft_core_server::RaftCoreServer, RaftCoreService},
  },
  client::node_client::RpcNodeClient,
  node::node::RaftNode,
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
    let host = std::env::var("HOST").expect("HOST is undefined");
    let id_node: u16 = std::env::var("ID_NODE")
      .expect("ID_NODE is undefined")
      .parse()
      .expect("Cannot parse the id node.");

    let nodes_string = std::env::var("ID_NODES").expect("ID_NODES is undefined");
    if nodes_string.is_empty() {
      panic!("ID_NODES is empty")
    }
    let node_ids: Vec<&str> = nodes_string.split(',').collect();
    if node_ids.len() % 2 == 1 {
      panic!("The cluster must have an odd number of nodes. ")
    }

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
      .expect("The NODE_CLIENT_TIMEOUT_MS isn't a number");
    RaftServerConfig {
      server_identification: id_node,
      peers_address: address_info,
      host_address: host,
      node_client_timeout: Duration::from_millis(timeout_ms),
    }
  }

  fn host(&self) -> String {
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

pub struct RaftServerRpc {
  node: Arc<Mutex<RaftNode>>,
  config: RaftServerConfig,
}

impl RaftServerRpc {
  pub fn new(config: RaftServerConfig) -> RaftServerRpc {
    // -- Node initialazing --//
    let node_client = RpcNodeClient::new(config.node_timeout());
    let node = RaftNode::new(
      config.server_id(),
      config.addresses(),
      node_client,
    );
    let arc_node = Arc::new(Mutex::new(node));

    RaftServerRpc {
      node: Arc::clone(&arc_node),
      config,
    }
  }

  pub async fn start(&self) {
    info!("Starting the Raft Server");
    tokio::spawn(background_tasks(Arc::clone(&self.node)));

    // -- Start the the web server -- //
    let addr = SocketAddr::from_str(&self.config.host()).unwrap();
    info!("[Listening on {addr}]");
    let core_service = RaftCoreService::new(Arc::clone(&self.node));
    let health_service = HealthService::new(Arc::clone(&self.node));
    let entry_service = EntryPointService::new(Arc::clone(&self.node));
    let reflection_service = tonic_reflection::server::Builder::configure()
      .register_encoded_file_descriptor_set(raft::FILE_DESCRIPTOR_SET)
      .register_encoded_file_descriptor_set(raft_api::FILE_DESCRIPTOR_SET)
      .build()
      .unwrap();

    Server::builder()
      .add_service(reflection_service)
      .add_service(RaftCoreServer::new(core_service))
      .add_service(HealthServer::new(health_service))
      .add_service(EntryPointServer::new(entry_service))
      .serve(addr)
      .await
      .unwrap();
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
