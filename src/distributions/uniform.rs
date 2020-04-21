use itertools::Itertools;
use num_integer::binomial;
use petgraph::{Graph, Undirected};
use rand::distributions::Distribution;
use rand::seq::IteratorRandom;
use rand::Rng;
use std::iter::FromIterator;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum UniformGraphError {
    #[error("too many edges")]
    TooManyEdges,
}

#[derive(Debug, Clone)]
pub struct UniformGraphDistribution {
    nodes: usize,
    edges: usize,
}

impl UniformGraphDistribution {
    /// Creates a new `UniformGraphDistribution` with `nodes` nodes, and `edges` edges.
    ///
    /// Will return an error if `edges > binomial(nodes, 2)`.
    ///
    /// # Example
    /// ```rust
    /// use random_graphs::prelude::*;
    /// use rand::prelude::*;
    ///
    /// let distribution = UniformGraphDistribution::new(4, 2).unwrap();
    ///
    /// // Generate a random graph
    /// let graph = distribution.sample(&mut thread_rng());
    /// assert_eq!(graph.node_count(), 4);
    /// assert_eq!(graph.edge_count(), 2);
    /// ```
    pub fn new(nodes: usize, edges: usize) -> Result<Self, UniformGraphError> {
        // Cannot have more than C(N, 2) edges in a graph on N edges.
        if edges > binomial(nodes, 2) {
            return Err(UniformGraphError::TooManyEdges);
        }

        Ok(Self { nodes, edges })
    }
}

impl Distribution<Graph<usize, (), Undirected>> for UniformGraphDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Graph<usize, (), Undirected> {
        let mut graph = Graph::with_capacity(self.nodes, self.edges);

        // Add all of our nodes to the graph
        let nodes = Vec::from_iter((0..self.nodes).map(|i| graph.add_node(i)));

        let chosen_edges = nodes
            .iter()
            .cartesian_product(nodes.iter())
            // Don't want to have self-loops, so filter out any (node, node) pairs
            .filter(|(node, other_node)| node != other_node)
            .choose_multiple(rng, self.edges);

        for (edge_start, edge_end) in chosen_edges {
            graph.add_edge(*edge_start, *edge_end, ());
        }

        graph
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use petgraph::prelude::EdgeRef;
    use rand::thread_rng;

    #[test]
    fn test_invalid_edge_count_causes_error() {
        // In an undirected graph on 4 nodes, there are at most 6 edges (count them, I dare you!)
        let distribution = UniformGraphDistribution::new(4, 6);
        assert!(distribution.is_ok());

        let distribution = UniformGraphDistribution::new(4, 7);
        assert_eq!(distribution.err(), Some(UniformGraphError::TooManyEdges));
    }

    #[test]
    fn test_uniform_graph_distribution() {
        let nodes = 4;
        let edges = 2;

        let distribution = UniformGraphDistribution::new(nodes, edges).unwrap();
        let mut rng = thread_rng();

        let mut edge_buckets = vec![vec![0; nodes]; nodes];

        for _ in 0..10000 {
            let graph = distribution.sample(&mut rng);
            assert_eq!(graph.node_count(), nodes);
            assert_eq!(graph.edge_count(), edges);

            for edge in graph.edge_references() {
                let src_index = edge.source().index();
                let tgt_index = edge.target().index();

                // Graph has no self loops
                assert_ne!(src_index, tgt_index);

                edge_buckets[src_index][tgt_index] += 1;
            }
        }

        let minimum_bucket_size = edge_buckets
            .iter()
            .enumerate()
            .map(|(index, inner_bucket)| {
                inner_bucket
                    .iter()
                    .enumerate()
                    .filter(|(inner_index, _)| *inner_index != index)
                    .min()
                    .unwrap()
                    .clone()
            })
            .map(|(_, inner_min)| *inner_min)
            .min()
            .unwrap();

        let maximum_bucket_size = edge_buckets
            .iter()
            .enumerate()
            .map(|(index, inner_bucket)| {
                inner_bucket
                    .iter()
                    .enumerate()
                    .filter(|(inner_index, _)| *inner_index != index)
                    .max()
                    .unwrap()
                    .clone()
            })
            .map(|(_, inner_max)| *inner_max)
            .max()
            .unwrap();

        // TODO: use the power of mathematics to determine the probability of obtaiing
        //  results at least as extreme as the one we did, assuming a uniform distribution
        //  with 10,000 samples.
        let relative_delta =
            ((maximum_bucket_size - minimum_bucket_size) as f32) / (minimum_bucket_size as f32);
        assert!(relative_delta < 0.10);
    }
}
