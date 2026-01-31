use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type NodeId = u64;
pub type EdgeId = u64;

#[derive(Clone)]
pub struct EnvironmentGraph {
    nodes: Arc<RwLock<HashMap<NodeId, Node>>>,
    edges: Arc<RwLock<HashMap<EdgeId, Edge>>>,
    adjacency: Arc<RwLock<HashMap<NodeId, Vec<EdgeId>>>>,
    next_node_id: Arc<RwLock<NodeId>>,
    next_edge_id: Arc<RwLock<EdgeId>>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
    pub properties: HashMap<String, String>,
    pub discovered_at: std::time::Instant,
    pub last_updated: std::time::Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Host {
        ip: String,
        hostname: Option<String>,
    },
    Service {
        host_id: NodeId,
        port: u16,
        protocol: String,
    },
    User {
        username: String,
        domain: Option<String>,
    },
    Credential {
        cred_type: String,
    },
    Vulnerability {
        cve: String,
        severity: f64,
    },
    File {
        path: String,
        host_id: NodeId,
    },
    Process {
        pid: u32,
        name: String,
        host_id: NodeId,
    },
    TrustRelationship {
        from_domain: String,
        to_domain: String,
    },
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
    pub edge_type: EdgeType,
    pub weight: f64,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    NetworkAccess,
    ServiceExploits,
    HasVulnerability,
    RequiresCredential,
    TrustsHost,
    RunsOn,
    OwnsFile,
    HasPrivilege,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct AttackPath {
    pub nodes: Vec<NodeId>,
    pub edges: Vec<EdgeId>,
    pub total_weight: f64,
    pub estimated_time: std::time::Duration,
    pub success_probability: f64,
}

impl EnvironmentGraph {
    pub fn new() -> Self {
        EnvironmentGraph {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            edges: Arc::new(RwLock::new(HashMap::new())),
            adjacency: Arc::new(RwLock::new(HashMap::new())),
            next_node_id: Arc::new(RwLock::new(1)),
            next_edge_id: Arc::new(RwLock::new(1)),
        }
    }

    pub async fn add_node(
        &self,
        node_type: NodeType,
        properties: HashMap<String, String>,
    ) -> NodeId {
        let node_id = {
            let mut next = self.next_node_id.write().await;
            let id = *next;
            *next += 1;
            id
        };

        let node = Node {
            id: node_id,
            node_type,
            properties,
            discovered_at: std::time::Instant::now(),
            last_updated: std::time::Instant::now(),
        };

        let mut nodes = self.nodes.write().await;
        nodes.insert(node_id, node);

        node_id
    }

    pub async fn add_edge(
        &self,
        from: NodeId,
        to: NodeId,
        edge_type: EdgeType,
        weight: f64,
    ) -> Result<EdgeId, String> {
        {
            let nodes = self.nodes.read().await;
            if !nodes.contains_key(&from) {
                return Err(format!("Source node {} not found", from));
            }
            if !nodes.contains_key(&to) {
                return Err(format!("Target node {} not found", to));
            }
        }

        let edge_id = {
            let mut next = self.next_edge_id.write().await;
            let id = *next;
            *next += 1;
            id
        };

        let edge = Edge {
            id: edge_id,
            from,
            to,
            edge_type,
            weight,
            properties: HashMap::new(),
        };

        {
            let mut edges = self.edges.write().await;
            edges.insert(edge_id, edge);
        }

        {
            let mut adjacency = self.adjacency.write().await;
            adjacency.entry(from).or_insert_with(Vec::new).push(edge_id);
        }

        Ok(edge_id)
    }

    pub async fn get_node(&self, node_id: NodeId) -> Option<Node> {
        let nodes = self.nodes.read().await;
        nodes.get(&node_id).cloned()
    }

    pub async fn get_edge(&self, edge_id: EdgeId) -> Option<Edge> {
        let edges = self.edges.read().await;
        edges.get(&edge_id).cloned()
    }

    pub async fn find_paths(
        &self,
        start: NodeId,
        goal: NodeId,
        max_depth: usize,
    ) -> Vec<AttackPath> {
        let mut paths = Vec::new();
        let mut queue = VecDeque::new();

        queue.push_back((start, vec![start], vec![], 0.0));

        while let Some((current, node_path, edge_path, total_weight)) = queue.pop_front() {
            if current == goal {
                paths.push(AttackPath {
                    nodes: node_path.clone(),
                    edges: edge_path.clone(),
                    total_weight,
                    estimated_time: std::time::Duration::from_secs((total_weight * 60.0) as u64),
                    success_probability: self.calculate_success_probability(&edge_path).await,
                });
                continue;
            }

            if node_path.len() > max_depth {
                continue;
            }

            let adjacency = self.adjacency.read().await;
            if let Some(outgoing) = adjacency.get(&current) {
                let edges = self.edges.read().await;

                for edge_id in outgoing {
                    if let Some(edge) = edges.get(edge_id) {
                        if !node_path.contains(&edge.to) {
                            let mut new_node_path = node_path.clone();
                            new_node_path.push(edge.to);

                            let mut new_edge_path = edge_path.clone();
                            new_edge_path.push(*edge_id);

                            queue.push_back((
                                edge.to,
                                new_node_path,
                                new_edge_path,
                                total_weight + edge.weight,
                            ));
                        }
                    }
                }
            }
        }

        paths.sort_by(|a, b| a.total_weight.partial_cmp(&b.total_weight).unwrap());
        paths
    }

    async fn calculate_success_probability(&self, edge_path: &[EdgeId]) -> f64 {
        if edge_path.is_empty() {
            return 1.0;
        }

        let edges = self.edges.read().await;
        let mut probability = 1.0;

        for edge_id in edge_path {
            if let Some(edge) = edges.get(edge_id) {
                let edge_success = 1.0 - (edge.weight / 10.0).min(0.9);
                probability *= edge_success;
            }
        }

        probability
    }

    pub async fn get_neighbors(&self, node_id: NodeId) -> Vec<NodeId> {
        let adjacency = self.adjacency.read().await;
        let edges = self.edges.read().await;

        if let Some(outgoing) = adjacency.get(&node_id) {
            outgoing
                .iter()
                .filter_map(|edge_id| edges.get(edge_id).map(|e| e.to))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub async fn find_nodes_by_type(
        &self,
        node_type_filter: impl Fn(&NodeType) -> bool,
    ) -> Vec<NodeId> {
        let nodes = self.nodes.read().await;
        nodes
            .values()
            .filter(|node| node_type_filter(&node.node_type))
            .map(|node| node.id)
            .collect()
    }

    pub async fn shortest_path(&self, start: NodeId, goal: NodeId) -> Option<AttackPath> {
        let paths = self.find_paths(start, goal, 20).await;
        paths.into_iter().next()
    }

    pub async fn update_node_properties(
        &self,
        node_id: NodeId,
        properties: HashMap<String, String>,
    ) -> Result<(), String> {
        let mut nodes = self.nodes.write().await;

        if let Some(node) = nodes.get_mut(&node_id) {
            for (key, value) in properties {
                node.properties.insert(key, value);
            }
            node.last_updated = std::time::Instant::now();
            Ok(())
        } else {
            Err(format!("Node {} not found", node_id))
        }
    }

    pub async fn remove_node(&self, node_id: NodeId) -> Result<(), String> {
        {
            let mut nodes = self.nodes.write().await;
            nodes
                .remove(&node_id)
                .ok_or_else(|| format!("Node {} not found", node_id))?;
        }

        {
            let mut adjacency = self.adjacency.write().await;
            adjacency.remove(&node_id);
        }

        {
            let mut edges = self.edges.write().await;
            edges.retain(|_, edge| edge.from != node_id && edge.to != node_id);
        }

        Ok(())
    }

    pub async fn node_count(&self) -> usize {
        let nodes = self.nodes.read().await;
        nodes.len()
    }

    pub async fn edge_count(&self) -> usize {
        let edges = self.edges.read().await;
        edges.len()
    }
}

pub struct EnvironmentDiscovery {
    graph: Arc<EnvironmentGraph>,
}

impl EnvironmentDiscovery {
    pub fn new(graph: Arc<EnvironmentGraph>) -> Self {
        EnvironmentDiscovery { graph }
    }

    pub async fn discover_network(&self, network_range: &str) -> Result<Vec<NodeId>, String> {
        let mut discovered = Vec::new();

        let hosts = self.scan_network(network_range).await?;

        for host_info in hosts {
            let props = HashMap::from([
                ("ip".to_string(), host_info.ip.clone()),
                ("status".to_string(), "alive".to_string()),
            ]);

            let node_id = self
                .graph
                .add_node(
                    NodeType::Host {
                        ip: host_info.ip,
                        hostname: host_info.hostname,
                    },
                    props,
                )
                .await;

            discovered.push(node_id);

            for service in host_info.services {
                let service_props = HashMap::from([
                    ("port".to_string(), service.port.to_string()),
                    ("protocol".to_string(), service.protocol.clone()),
                ]);

                let service_id = self
                    .graph
                    .add_node(
                        NodeType::Service {
                            host_id: node_id,
                            port: service.port,
                            protocol: service.protocol,
                        },
                        service_props,
                    )
                    .await;

                self.graph
                    .add_edge(node_id, service_id, EdgeType::RunsOn, 0.1)
                    .await?;
            }
        }

        Ok(discovered)
    }

    async fn scan_network(&self, _range: &str) -> Result<Vec<HostInfo>, String> {
        Ok(Vec::new())
    }
}

struct HostInfo {
    ip: String,
    hostname: Option<String>,
    services: Vec<ServiceInfo>,
}

struct ServiceInfo {
    port: u16,
    protocol: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_graph_basic() {
        let graph = EnvironmentGraph::new();

        let node1 = graph
            .add_node(
                NodeType::Host {
                    ip: "192.168.1.1".to_string(),
                    hostname: None,
                },
                HashMap::new(),
            )
            .await;

        let node2 = graph
            .add_node(
                NodeType::Host {
                    ip: "192.168.1.2".to_string(),
                    hostname: None,
                },
                HashMap::new(),
            )
            .await;

        graph
            .add_edge(node1, node2, EdgeType::NetworkAccess, 1.0)
            .await
            .unwrap();

        assert_eq!(graph.node_count().await, 2);
        assert_eq!(graph.edge_count().await, 1);
    }

    #[tokio::test]
    async fn test_pathfinding() {
        let graph = EnvironmentGraph::new();

        let n1 = graph
            .add_node(
                NodeType::Host {
                    ip: "192.168.1.1".to_string(),
                    hostname: None,
                },
                HashMap::new(),
            )
            .await;
        let n2 = graph
            .add_node(
                NodeType::Host {
                    ip: "192.168.1.2".to_string(),
                    hostname: None,
                },
                HashMap::new(),
            )
            .await;
        let n3 = graph
            .add_node(
                NodeType::Host {
                    ip: "192.168.1.3".to_string(),
                    hostname: None,
                },
                HashMap::new(),
            )
            .await;

        graph
            .add_edge(n1, n2, EdgeType::NetworkAccess, 1.0)
            .await
            .unwrap();
        graph
            .add_edge(n2, n3, EdgeType::NetworkAccess, 1.0)
            .await
            .unwrap();

        let path = graph.shortest_path(n1, n3).await;
        assert!(path.is_some());

        let path = path.unwrap();
        assert_eq!(path.nodes.len(), 3);
        assert_eq!(path.nodes[0], n1);
        assert_eq!(path.nodes[2], n3);
    }
}
