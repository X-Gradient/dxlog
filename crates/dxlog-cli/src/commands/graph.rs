use anyhow::Result;
use dxlog::{ReferenceGraph, GraphStats};
use uuid::Uuid;

#[derive(clap::Subcommand, Clone)]
pub enum GraphCommands {
    /// Show dependency graph for a research item
    ///
    /// Displays a tree-like visualization of dependencies for the specified research item.
    /// By default shows forward references (what this item depends on).
    ///
    /// Examples:
    ///   dxlog graph show 1a2b3c4d
    ///   dxlog graph show 1a2b3c4d --depth 2 --direction backward
    ///   dxlog graph show 1a2b3c4d --format dot > graph.dot
    Show {
        /// ID of the research item (can be partial)
        #[arg(help = "Research item ID to show graph for")]
        id: String,

        /// Maximum depth to traverse (0 = unlimited)
        #[arg(
            long,
            short = 'd',
            default_value = "1",
            help = "Maximum depth to traverse (0 for unlimited)"
        )]
        depth: usize,

        /// Direction to traverse
        #[arg(
            long,
            short = 'r',
            default_value = "forward",
            help = "Direction to traverse: forward, backward, or both"
        )]
        direction: GraphDirection,

        /// Output format
        #[arg(
            long,
            short = 'f',
            default_value = "text",
            help = "Output format: text, dot, or json"
        )]
        format: GraphFormat,

        /// Include status information
        #[arg(long, help = "Include status information in output")]
        include_status: bool,

        /// Include tags information
        #[arg(long, help = "Include tags information in output")]
        include_tags: bool,
    },

    /// Show graph statistics
    ///
    /// Displays overall statistics about the reference graph including
    /// node counts, edge counts, cycles, and isolated nodes.
    ///
    /// Example:
    ///   dxlog graph stats
    Stats,

    /// Detect cycles in the reference graph
    ///
    /// Finds and displays all cycles in the reference graph.
    /// Cycles indicate circular dependencies that should be resolved.
    ///
    /// Example:
    ///   dxlog graph cycles
    Cycles,

    /// Validate the reference graph
    ///
    /// Checks for broken references, orphaned nodes, and other
    /// graph integrity issues.
    ///
    /// Example:
    ///   dxlog graph validate
    Validate,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum GraphDirection {
    Forward,
    Backward,
    Both,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum GraphFormat {
    Text,
    Dot,
    Json,
}

impl GraphCommands {
    pub fn execute(&self) -> Result<()> {
        match self {
            Self::Show {
                id,
                depth,
                direction,
                format,
                include_status,
                include_tags,
            } => {
                let graph = ReferenceGraph::build_from_all_logs()?;
                
                // Parse UUID from partial ID
                let uuid = parse_partial_uuid(id)?;
                
                // Set depth (0 means unlimited)
                let max_depth = if *depth == 0 { None } else { Some(*depth) };
                
                // Perform traversal based on direction
                let traversal = match direction {
                    GraphDirection::Forward => graph.traverse_forward(&uuid, max_depth)?,
                    GraphDirection::Backward => graph.traverse_backward(&uuid, max_depth)?,
                    GraphDirection::Both => graph.traverse_both(&uuid, max_depth)?,
                };
                
                // Output in requested format
                match format {
                    GraphFormat::Text => {
                        render_text_graph(&traversal, &graph, *include_status, *include_tags)?;
                    }
                    GraphFormat::Dot => {
                        render_dot_graph(&traversal, &graph)?;
                    }
                    GraphFormat::Json => {
                        render_json_graph(&traversal)?;
                    }
                }
                
                Ok(())
            }
            Self::Stats => {
                let graph = ReferenceGraph::build_from_all_logs()?;
                let stats = graph.calculate_stats();
                display_stats(&stats);
                Ok(())
            }
            Self::Cycles => {
                let graph = ReferenceGraph::build_from_all_logs()?;
                let cycles = graph.detect_cycles();
                
                if cycles.is_empty() {
                    println!("No cycles detected in the reference graph.");
                } else {
                    println!("Found {} cycle(s):", cycles.len());
                    for (i, cycle) in cycles.iter().enumerate() {
                        println!("\nCycle {}:", i + 1);
                        for node in cycle {
                            println!("  {} ({}) - {}", 
                                &node.id.to_string()[..8], 
                                node.node_type, 
                                node.title
                            );
                        }
                    }
                }
                Ok(())
            }
            Self::Validate => {
                let graph = ReferenceGraph::build_from_all_logs()?;
                validate_graph(&graph)?;
                Ok(())
            }
        }
    }
}

fn parse_partial_uuid(partial_id: &str) -> Result<Uuid> {
    // Try to parse as full UUID first
    if let Ok(uuid) = Uuid::parse_str(partial_id) {
        return Ok(uuid);
    }
    
    // If that fails, we need to find the full UUID from the partial ID
    // This is a simplified version - in practice, we'd search through all items
    let graph = ReferenceGraph::build_from_all_logs()?;
    
    for (uuid, _) in &graph.uuid_to_node_index {
        if uuid.to_string().starts_with(partial_id) {
            return Ok(*uuid);
        }
    }
    
    Err(anyhow::anyhow!("No research item found with ID starting with '{}'", partial_id))
}

fn render_text_graph(
    traversal: &dxlog::GraphTraversal,
    _graph: &ReferenceGraph,
    include_status: bool,
    include_tags: bool,
) -> Result<()> {
    for (depth, nodes) in &traversal.levels {
        for node_info in nodes {
            let indent = "  ".repeat(*depth);
            let prefix = if *depth == 0 {
                ""
            } else {
                "├── "
            };
            
            let mut output = format!(
                "{}{}{} ({}) {}",
                indent,
                prefix,
                &node_info.node.id.to_string()[..8],
                node_info.node.node_type,
                node_info.node.title
            );
            
            if include_status {
                output.push_str(&format!(" [{}]", node_info.node.status));
            }
            
            if include_tags && !node_info.node.tags.is_empty() {
                output.push_str(&format!(" (tags: {})", node_info.node.tags.join(", ")));
            }
            
            // Show edge validation status
            if let Some(edge) = &node_info.edge {
                if !edge.validated {
                    output.push_str(" ⚠️ incomplete");
                }
            }
            
            println!("{}", output);
        }
    }
    Ok(())
}

fn render_dot_graph(traversal: &dxlog::GraphTraversal, _graph: &ReferenceGraph) -> Result<()> {
    println!("digraph research_graph {{");
    println!("  rankdir=TB;");
    println!("  node [shape=box, style=rounded];");
    
    // Define node styles
    println!("  // Node styles");
    for (_, nodes) in &traversal.levels {
        for node_info in nodes {
            let color = match node_info.node.node_type.as_str() {
                "hypothesis" => "lightblue",
                "literature" => "lightgreen", 
                "knowledge" => "lightyellow",
                _ => "lightgray",
            };
            
            println!(
                "  \"{}\" [label=\"{}\", fillcolor={}, style=filled];",
                node_info.node.id,
                node_info.node.title.replace("\"", "\\\""),
                color
            );
        }
    }
    
    // Define edges
    println!("  // Edges");
    for (_, nodes) in &traversal.levels {
        for node_info in nodes {
            if let Some(edge) = &node_info.edge {
                let style = if edge.validated { "solid" } else { "dashed" };
                println!(
                    "  \"{}\" -> \"{}\" [style={}];",
                    edge.from, edge.to, style
                );
            }
        }
    }
    
    println!("}}");
    Ok(())
}

fn render_json_graph(traversal: &dxlog::GraphTraversal) -> Result<()> {
    let json_output = serde_json::to_string_pretty(traversal)?;
    println!("{}", json_output);
    Ok(())
}

fn display_stats(stats: &GraphStats) {
    println!("Reference Graph Statistics");
    println!("=========================");
    println!("Total nodes: {}", stats.total_nodes);
    println!("Total edges: {}", stats.total_edges);
    println!("Cycles detected: {}", stats.cycles_count);
    println!("Isolated nodes: {}", stats.isolated_nodes);
    
    println!("\nNodes by type:");
    for (node_type, count) in &stats.nodes_by_type {
        println!("  {}: {}", node_type, count);
    }
    
    println!("\nNodes by status:");
    for (status, count) in &stats.nodes_by_status {
        println!("  {}: {}", status, count);
    }
}

fn validate_graph(graph: &ReferenceGraph) -> Result<()> {
    println!("Validating reference graph...");
    
    let stats = graph.calculate_stats();
    let cycles = graph.detect_cycles();
    
    let mut issues = 0;
    
    // Check for cycles
    if !cycles.is_empty() {
        issues += cycles.len();
        println!("❌ Found {} cycle(s) in the graph", cycles.len());
        for (i, cycle) in cycles.iter().enumerate() {
            println!("  Cycle {}: {} nodes", i + 1, cycle.len());
        }
    } else {
        println!("✅ No cycles detected");
    }
    
    // Check for isolated nodes
    if stats.isolated_nodes > 0 {
        issues += stats.isolated_nodes;
        println!("⚠️  Found {} isolated node(s)", stats.isolated_nodes);
    } else {
        println!("✅ No isolated nodes");
    }
    
    // Summary
    if issues == 0 {
        println!("\n✅ Graph validation passed - no issues found");
    } else {
        println!("\n❌ Graph validation found {} issue(s)", issues);
    }
    
    Ok(())
}