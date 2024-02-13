use std::{
  collections::HashMap,
  net::SocketAddr,
  str::FromStr,
  sync::{Arc, Mutex},
  thread,
  time::Duration,
};

use axum::Router;
use log::error;
use log::info;

use crate::{api, node::node::RaftNode};

#[derive(Clone)]
pub struct RaftServerConfig {
  server_identification: u16,
  peers_address: HashMap<u16, SocketAddr>,
  host_address: String,
  waiting: Duration,
}

impl RaftServerConfig {
  pub fn new() -> RaftServerConfig {
    dotenv::dotenv().ok();
    // -- Configuration --//
    let host = std::env::var("HOST").expect("Host undefined");
    let id_node: u16 = std::env::var("ID_NODE")
      .expect("NodeID unfefined")
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
    RaftServerConfig {
      server_identification: id_node,
      peers_address: address_info,
      host_address: host,
      waiting: Duration::from_secs(5),
    }
  }

  fn startup_waiting(&self) -> Duration {
    self.waiting.clone()
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
}

pub struct RaftServer {
  node: Arc<Mutex<RaftNode>>,
  router: Router,
  config: RaftServerConfig,
}

impl RaftServer {
  pub fn new(config: &RaftServerConfig) -> RaftServer {
    // -- Node initialazing --//
    let node = RaftNode::new(config.server_id(), config.addresses());
    // dbg!(node.clone());
    let arc_node = Arc::new(Mutex::new(node));

    // -- Routes --//
    let router = Router::new().merge(api::api::routes(Arc::clone(&arc_node)));
    RaftServer {
      node: Arc::clone(&arc_node),
      router,
      config: config.clone(),
    }
  }

  pub fn routes(&self) -> Router {
    self.router.clone()
  }

  pub fn start(&mut self) {
    info!("Starting the Raft Server");
    tokio::spawn(background_tasks(Arc::clone(&self.node)));

    // -- Wait to others nodes in the network some random time --//
    let waiting_time = self.config.startup_waiting();
    let waiting_as_secs = waiting_time.as_secs();
    info!("Wating {waiting_as_secs} seconds for other nodes ...");
    thread::sleep(waiting_time);

    self.node.lock().unwrap().wait_to_another_election();
  }
}

async fn background_tasks(node: Arc<Mutex<RaftNode>>) {
  loop {
    match node.lock() {
      Ok(mut lock_node) => {
        lock_node.handle_timeout();
        lock_node.broadcast_heartbeat();
      },
      Err(e) => {
        error!("Cannot lock the resource");
        panic!("Not good {e}")
      },
    }
  }
}
