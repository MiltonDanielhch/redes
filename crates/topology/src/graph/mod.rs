//! Ubicación: `crates/topology/src/graph/mod.rs`
//!
//! Descripción: Estructuras de grafo para topología de red.
//!
//! ADRs relacionados: 0020 (Monitoreo Regional)

use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    pub id: Uuid,
    pub device_id: Uuid,
    pub node_type: NodeType,
    pub connections: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Switch,
    Router,
    Firewall,
    Server,
    Ups,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkGraph {
    pub nodes: Vec<NetworkNode>,
    pub edges: Vec<(Uuid, Uuid)>,
}

impl NetworkGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: NetworkNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, from: Uuid, to: Uuid) {
        self.edges.push((from, to));
    }
}