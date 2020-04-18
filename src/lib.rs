use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum GraphError<I: Debug, E: EdgeLike<I>> {
    #[error("node `{0:?}` was not found in graph")]
    MissingNode(I),
    #[error("edge `{0:?}` is invalid")]
    InvalidEdge(E),
}

pub trait EdgeLike<I: Debug>: Debug {
    fn source(&self) -> &I;
    fn target(&self) -> &I;
    fn key(&self) -> Option<usize>;
}

pub trait GraphLike<I: Debug, E: EdgeLike<I>> {
    /// Add a node to this graph
    fn add_node(&mut self, node: I) -> bool;

    /// Does this graph contain the given node?
    fn has_node(&self, node: &I) -> bool;

    /// Add an edge to the graph.
    fn add_edge(&mut self, edge: E) -> Result<bool, GraphError<I, E>>;

    /// Does this graph contain the given edge?
    fn has_edge(&self, edge: &E) -> bool;

    /// Returns `true` if this graph is directed.
    fn is_directed(&self) -> bool;

    /// Returns `true` if this graph is undirected.
    fn is_undirected(&self) -> bool {
        !self.is_directed()
    }

    /// Returns an iterator over each of the nodes in the graph.
    fn node_iter(&self) -> Box<dyn Iterator<Item = &I> + '_>;

    /// Returns an iterator over each of the edges in the graph.
    fn edge_iter(&self) -> Box<dyn Iterator<Item = &E> + '_>;
}

#[derive(Debug, Clone)]
pub struct DumbEdge {
    source: usize,
    target: usize,
    key: Option<usize>,
}

impl DumbEdge {
    pub fn new(source: usize, target: usize) -> DumbEdge {
        Self {
            source,
            target,
            key: None,
        }
    }
}

impl EdgeLike<usize> for DumbEdge {
    fn source(&self) -> &usize {
        &self.source
    }

    fn target(&self) -> &usize {
        &self.target
    }

    fn key(&self) -> Option<usize> {
        self.key
    }
}

#[derive(Debug, Clone)]
pub struct DumbGraph {
    adj: HashMap<usize, HashSet<usize>>,
    edges: Vec<DumbEdge>,
}

impl DumbGraph {
    pub fn new() -> Self {
        Self {
            adj: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn is_valid_edge(&self, edge: &DumbEdge) -> bool {
        self.has_node(edge.source()) && self.has_node(edge.target())
    }
}

impl GraphLike<usize, DumbEdge> for DumbGraph {
    fn add_node(&mut self, node: usize) -> bool {
        if let Some(adj_set) = self.adj.get_mut(&node) {
            adj_set.insert(node)
        } else {
            self.adj.insert(node, {
                let mut hs = HashSet::new();
                hs.insert(node);
                hs
            });
            true
        }
    }

    fn has_node(&self, node: &usize) -> bool {
        self.adj.contains_key(node)
    }

    fn add_edge(&mut self, edge: DumbEdge) -> Result<bool, GraphError<usize, DumbEdge>> {
        if !self.is_valid_edge(&edge) {
            return Err(GraphError::InvalidEdge(edge));
        }

        let edge_was_inserted = self.adj.get_mut(&edge.source).unwrap().insert(edge.target);
        self.edges.push(edge);

        Ok(edge_was_inserted)
    }

    fn has_edge(&self, edge: &DumbEdge) -> bool {
        self.is_valid_edge(edge) && self.adj[&edge.source].contains(&edge.target)
    }

    fn is_directed(&self) -> bool {
        true
    }

    fn node_iter(&self) -> Box<dyn Iterator<Item = &usize> + '_> {
        Box::new(self.adj.keys())
    }

    fn edge_iter(&self) -> Box<dyn Iterator<Item = &DumbEdge> + '_> {
        Box::new(self.edges.iter())
    }
}

#[test]
fn test_dumb_graph() {
    let mut g = DumbGraph::new();
    g.add_node(3);
    g.add_node(7);
    g.add_edge(DumbEdge::new(3, 7));

    {
        // Check that node iteration works as expected
        let mut node_iter = g.node_iter();
        assert!(node_iter.next().is_some());
        assert!(node_iter.next().is_some());
        assert_eq!(node_iter.next(), None);
    }

    {
        // Check that edge iteration works as expected
        let mut edge_iter = g.edge_iter();
        let next_edge = edge_iter.next().unwrap();
        assert_eq!(next_edge.source, 3);
        assert_eq!(next_edge.target, 7);
        assert!(edge_iter.next().is_none());
    }
}