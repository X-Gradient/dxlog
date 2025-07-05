// crates/dxlog/src/graph.rs
use crate::{
    load_config, research_log::ResearchLog, utils::BaseLog, HypothesisManager, KnowledgeManager, LiteratureManager,
};
use anyhow::Result;
use indexmap::IndexMap;
use petgraph::{Graph, Direction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: Uuid,
    pub title: String,
    pub node_type: String,
    pub status: String,
    pub tags: Vec<String>,
    pub date: String,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub validated: bool, // Whether the target is in a complete state
}

#[derive(Debug)]
pub struct ReferenceGraph {
    pub graph: Graph<GraphNode, GraphEdge>,
    pub uuid_to_node_index: HashMap<Uuid, petgraph::graph::NodeIndex>,
}

impl ReferenceGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            uuid_to_node_index: HashMap::new(),
        }
    }

    pub fn build_from_all_logs() -> Result<Self> {
        let mut graph = Self::new();
        
        let config = load_config()?;
        let h_manager = HypothesisManager::new(config.clone());
        let l_manager = LiteratureManager::new(config.clone());
        let k_manager = KnowledgeManager::new(config.clone());

        // Add all nodes first
        let hypotheses = h_manager.manager.log_manager.list_logs(None, None)?;
        for hypothesis in hypotheses {
            let base = hypothesis.base();
            let node = GraphNode {
                id: base.id,
                title: base.title.clone(),
                node_type: "hypothesis".to_string(),
                status: hypothesis.status().to_string(),
                tags: base.tags.iter().cloned().collect(),
                date: base.date.clone(),
                author: base.created_by.name.clone(),
            };
            graph.add_node(node);
        }

        let literature_items = l_manager.manager.log_manager.list_logs(None, None)?;
        for literature in literature_items {
            let base = literature.base();
            let node = GraphNode {
                id: base.id,
                title: base.title.clone(),
                node_type: "literature".to_string(),
                status: literature.status().to_string(),
                tags: base.tags.iter().cloned().collect(),
                date: base.date.clone(),
                author: base.created_by.name.clone(),
            };
            graph.add_node(node);
        }

        let knowledge_items = k_manager.manager.log_manager.list_logs(None, None)?;
        for knowledge in knowledge_items {
            let base = knowledge.base();
            let node = GraphNode {
                id: base.id,
                title: base.title.clone(),
                node_type: "knowledge".to_string(),
                status: knowledge.status().to_string(),
                tags: base.tags.iter().cloned().collect(),
                date: base.date.clone(),
                author: base.created_by.name.clone(),
            };
            graph.add_node(node);
        }

        // Add all edges
        let hypotheses = h_manager.manager.log_manager.list_logs(None, None)?;
        for hypothesis in hypotheses {
            let base = hypothesis.base();
            graph.add_edges_for_log(base)?;
        }

        let literature_items = l_manager.manager.log_manager.list_logs(None, None)?;
        for literature in literature_items {
            let base = literature.base();
            graph.add_edges_for_log(base)?;
        }

        let knowledge_items = k_manager.manager.log_manager.list_logs(None, None)?;
        for knowledge in knowledge_items {
            let base = knowledge.base();
            graph.add_edges_for_log(base)?;
        }

        Ok(graph)
    }

    fn add_node(&mut self, node: GraphNode) {
        let node_index = self.graph.add_node(node.clone());
        self.uuid_to_node_index.insert(node.id, node_index);
    }

    fn add_edges_for_log(&mut self, base_log: &BaseLog) -> Result<()> {
        if let Some(&source_index) = self.uuid_to_node_index.get(&base_log.id) {
            for &target_uuid in &base_log.references {
                if let Some(&target_index) = self.uuid_to_node_index.get(&target_uuid) {
                    // Check if target is validated (in complete state)
                    let target_node = &self.graph[target_index];
                    let validated = is_status_complete(&target_node.node_type, &target_node.status);
                    
                    let edge = GraphEdge {
                        from: base_log.id,
                        to: target_uuid,
                        validated,
                    };
                    
                    self.graph.add_edge(source_index, target_index, edge);
                }
            }
        }
        Ok(())
    }

    pub fn get_node_by_id(&self, id: &Uuid) -> Option<&GraphNode> {
        if let Some(&node_index) = self.uuid_to_node_index.get(id) {
            Some(&self.graph[node_index])
        } else {
            None
        }
    }

    pub fn traverse_forward(&self, start_id: &Uuid, max_depth: Option<usize>) -> Result<GraphTraversal> {
        self.traverse(start_id, Direction::Outgoing, max_depth)
    }

    pub fn traverse_backward(&self, start_id: &Uuid, max_depth: Option<usize>) -> Result<GraphTraversal> {
        self.traverse(start_id, Direction::Incoming, max_depth)
    }

    pub fn traverse_both(&self, start_id: &Uuid, max_depth: Option<usize>) -> Result<GraphTraversal> {
        let mut forward = self.traverse_forward(start_id, max_depth)?;
        let backward = self.traverse_backward(start_id, max_depth)?;
        
        // Merge the results
        for (depth, nodes) in backward.levels {
            forward.levels.entry(depth).or_insert_with(Vec::new).extend(nodes);
        }
        
        Ok(forward)
    }

    fn traverse(&self, start_id: &Uuid, direction: Direction, max_depth: Option<usize>) -> Result<GraphTraversal> {
        let start_index = self.uuid_to_node_index.get(start_id)
            .ok_or_else(|| anyhow::anyhow!("Node with ID {} not found", start_id))?;

        let mut traversal = GraphTraversal::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        
        // Add the starting node at depth 0
        let start_node = &self.graph[*start_index];
        traversal.levels.entry(0).or_insert_with(Vec::new).push(TraversalNode {
            node: start_node.clone(),
            depth: 0,
            edge: None,
        });
        
        queue.push_back((*start_index, 0));
        visited.insert(*start_index);

        while let Some((current_index, depth)) = queue.pop_front() {
            // Check depth limit
            if let Some(max_d) = max_depth {
                if depth >= max_d {
                    continue;
                }
            }

            // Traverse neighbors
            let neighbors: Vec<_> = self.graph.neighbors_directed(current_index, direction).collect();
            for neighbor_index in neighbors {
                if !visited.contains(&neighbor_index) {
                    visited.insert(neighbor_index);
                    
                    let neighbor_node = &self.graph[neighbor_index];
                    let edge_index = self.graph.find_edge(current_index, neighbor_index)
                        .or_else(|| self.graph.find_edge(neighbor_index, current_index));
                    
                    let edge = edge_index.map(|ei| self.graph[ei].clone());
                    
                    traversal.levels.entry(depth + 1).or_insert_with(Vec::new).push(TraversalNode {
                        node: neighbor_node.clone(),
                        depth: depth + 1,
                        edge,
                    });
                    
                    queue.push_back((neighbor_index, depth + 1));
                }
            }
        }

        Ok(traversal)
    }

    pub fn detect_cycles(&self) -> Vec<Vec<GraphNode>> {
        let mut cycles = Vec::new();
        
        // Use DFS to detect cycles
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();
        
        for node_index in self.graph.node_indices() {
            if !visited.contains(&node_index) {
                if let Some(cycle) = self.dfs_cycle_detection(node_index, &mut visited, &mut rec_stack) {
                    cycles.push(cycle);
                }
            }
        }
        
        cycles
    }
    
    fn dfs_cycle_detection(
        &self,
        node: petgraph::graph::NodeIndex,
        visited: &mut std::collections::HashSet<petgraph::graph::NodeIndex>,
        rec_stack: &mut std::collections::HashSet<petgraph::graph::NodeIndex>,
    ) -> Option<Vec<GraphNode>> {
        visited.insert(node);
        rec_stack.insert(node);
        
        for neighbor in self.graph.neighbors(node) {
            if !visited.contains(&neighbor) {
                if let Some(cycle) = self.dfs_cycle_detection(neighbor, visited, rec_stack) {
                    return Some(cycle);
                }
            } else if rec_stack.contains(&neighbor) {
                // Found a cycle
                let cycle = vec![self.graph[neighbor].clone()];
                return Some(cycle);
            }
        }
        
        rec_stack.remove(&node);
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalNode {
    pub node: GraphNode,
    pub depth: usize,
    pub edge: Option<GraphEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphTraversal {
    pub levels: IndexMap<usize, Vec<TraversalNode>>,
}

impl GraphTraversal {
    fn new() -> Self {
        Self {
            levels: IndexMap::new(),
        }
    }
}

fn is_status_complete(node_type: &str, status: &str) -> bool {
    match node_type {
        "hypothesis" => matches!(status, "proven" | "disproven" | "inconclusive"),
        "literature" => status == "completed",
        "knowledge" => status == "published",
        _ => false,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub nodes_by_type: HashMap<String, usize>,
    pub nodes_by_status: HashMap<String, usize>,
    pub cycles_count: usize,
    pub isolated_nodes: usize,
}

impl ReferenceGraph {
    pub fn calculate_stats(&self) -> GraphStats {
        let mut nodes_by_type = HashMap::new();
        let mut nodes_by_status = HashMap::new();
        let mut isolated_count = 0;

        for node_index in self.graph.node_indices() {
            let node = &self.graph[node_index];
            
            *nodes_by_type.entry(node.node_type.clone()).or_insert(0) += 1;
            *nodes_by_status.entry(node.status.clone()).or_insert(0) += 1;
            
            // Check if node is isolated (no incoming or outgoing edges)
            if self.graph.neighbors_undirected(node_index).count() == 0 {
                isolated_count += 1;
            }
        }

        let cycles = self.detect_cycles();

        GraphStats {
            total_nodes: self.graph.node_count(),
            total_edges: self.graph.edge_count(),
            nodes_by_type,
            nodes_by_status,
            cycles_count: cycles.len(),
            isolated_nodes: isolated_count,
        }
    }
}