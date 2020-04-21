use petgraph::{Graph, Undirected};
use rand::distributions::{Bernoulli, Distribution};
use rand::Rng;
use std::iter::FromIterator;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum BinomialGraphError {
    #[error("invalid parameter `p` = {0}, should be 0 <= `p` <= 1")]
    InvalidProbability(f64),
}

#[derive(Debug, Clone)]
pub struct BinomialGraphDistribution {
    nodes: usize,
    p: f64,
}

impl BinomialGraphDistribution {
    /// Creates a new `BinomialGraphDistribution` with `nodes` nodes, and where up to
    /// `binomial(nodes, 2)` edges are inserted independently with probability `p`.
    ///
    /// Will return an error if `p < 0` or `p > 1`.
    ///
    /// # Example
    /// ```rust
    /// use random_graphs::prelude::*;
    /// use rand::prelude::*;
    ///
    /// let distribution = BinomialGraphDistribution::new(4, 0.25).unwrap();
    ///
    /// // Generate a random graph
    /// let graph = distribution.sample(&mut thread_rng());
    /// assert_eq!(graph.node_count(), 4);
    /// ```
    pub fn new(nodes: usize, p: f64) -> Result<Self, BinomialGraphError> {
        // Probability must be between 0 and 1.
        if p < 0.0 || p > 1.0 {
            return Err(BinomialGraphError::InvalidProbability(p));
        }

        Ok(Self { nodes, p })
    }
}

impl Distribution<Graph<usize, (), Undirected>> for BinomialGraphDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Graph<usize, (), Undirected> {
        // Expected number of edges is binomial(n, 2) * p
        let mut graph = Graph::new_undirected();

        let nodes = Vec::from_iter((0..self.nodes).map(|index| graph.add_node(index)));

        // Unwrap is fine here because we've already verified that 0 <= self.p <= 1.
        let bernoulli = Bernoulli::new(self.p).unwrap();

        for (index, start_node) in nodes.iter().enumerate() {
            for end_node in nodes.iter().skip(index + 1) {
                if bernoulli.sample(rng) {
                    graph.add_edge(start_node.clone(), end_node.clone(), ());
                }
            }
        }

        graph
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_invalid_p_causes_error() {
        // Negative value should cause an error
        let distribution = BinomialGraphDistribution::new(4, -0.05);
        assert_eq!(distribution.err(), Some(BinomialGraphError::InvalidProbability(-0.05)));

        // A couple of p-values that should be fine
        for acceptable_p in &[0.0, 0.05, 0.4, 0.77, 0.33, 0.999, 1.0] {
            let distribution = BinomialGraphDistribution::new(4, *acceptable_p);
            assert!(distribution.is_ok());
        }

        // A value greater than 1 should cause an error
        let distribution = BinomialGraphDistribution::new(4, 1.01);
        assert_eq!(distribution.err(), Some(BinomialGraphError::InvalidProbability(1.01)));
    }

    #[test]
    fn test_binomial_graph_distribution() {
        // Given 9 nodes, there are 36 possible edges.  Using `p = 1/6` the expected
        // number of edges in our graph is 6.
        let nodes = 9;
        let p = 1.0 / 6.0;

        let distribution = BinomialGraphDistribution::new(nodes, p).unwrap();
        let mut rng = thread_rng();

        let iteration_count = 10000;

        // Count the number of edges across 10,000 generations
        let edge_count : usize = (0..iteration_count)
            .map(|_| distribution.sample(&mut rng).edge_count())
            .sum();

        let average_number_of_edges = (edge_count as f64) / (iteration_count as f64);

        // TODO: do some mathematics here to figure out what a reasonable relative tolerance
        //  is here for 10,000 samples (CLT, LOLN).
        let relative_tolerance = (average_number_of_edges - 6.0) / 6.0;
        assert!(relative_tolerance < 0.01);
    }
}