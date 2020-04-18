use thiserror::Error;
use rand::Rng;
pub use petgraph::Graph as Graph;
use petgraph::dot::{Dot, Config};

#[derive(Debug, Error)]
pub enum RandomTreeError {
    #[error("cannot generate tree with `average_leaves` = `{0}`")]
    InvalidAverageLeaves(usize),
    #[error("cannot generate tree with `desired_size` = `{0}`")]
    InvalidDesiredSize(usize),
}

struct RandomTreeOptions {
    desired_size: usize,
    average_leaves: usize,
}

impl RandomTreeOptions {
    pub fn new(desired_size: usize, average_leaves: usize) -> Result<Self, RandomTreeError> {
        if desired_size == 0 {
            return Err(RandomTreeError::InvalidDesiredSize(desired_size));
        }
        if average_leaves == 0 {
            return Err(RandomTreeError::InvalidAverageLeaves(average_leaves));
        }

        Ok(Self {
            desired_size,
            average_leaves,
        })
    }
}

// Create a random tree
fn random_tree(options: RandomTreeOptions) -> Result<Graph<usize, usize>, RandomTreeError> {
    // TODO: could be generic over the way we label nodes / edges
    // TODO: could allow different options based on the originating node "type"
    // TODO: could do a lot of things
    // TODO: something like a node classifier? with distributions on each classification
    // nice..
    let mut tree = Graph::<usize, usize>::new();
    let mut rand = rand::thread_rng();

    let mut boundary = vec![tree.add_node(0)];
    let mut counter = 0;

    while tree.node_count() < options.desired_size {
        if let Some(boundary_node) = boundary.pop() {
            let number_of_leaves = rand.gen_range(1, options.average_leaves);

            for _ in 0..number_of_leaves {
                counter += 1;
                let node = tree.add_node(counter);
                tree.add_edge(boundary_node, node, 1);
                boundary.push(node);
            }
        } else {
            // No more nodes on the boundary, but still don't have enough nodes in the tree
            // -> something went wrong.
            panic!("empty boundary: found {} nodes, required {} nodes");
        }
    }

    Ok(tree)
}

#[test]
fn test_something() {
    let options = RandomTreeOptions::new(50, 8).unwrap();
    println!("{:?}", Dot::with_config(&random_tree(options).unwrap(), &[Config::EdgeNoLabel]));

    panic!();
}