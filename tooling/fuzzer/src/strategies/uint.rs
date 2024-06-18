use proptest::{
    strategy::{NewTree, Strategy},
    test_runner::TestRunner,
};
use rand::Rng;

/// Value tree for unsigned ints (up to u128).
/// The strategy combines 2 different strategies, each assigned a specific weight:
/// 1. Generate purely random value in a range. This will first choose bit size uniformly (up `bits`
///    param). Then generate a value for this bit size.
/// 2. Generate a random value around the edges (+/- 3 around 0 and max possible value)
#[derive(Debug)]
pub struct UintStrategy {
    /// Bit size of uint (e.g. 128)
    bits: usize,

    /// The weight for edge cases (+/- 3 around 0 and max possible value)
    edge_weight: usize,
    /// The weight for purely random values
    random_weight: usize,
}

impl UintStrategy {
    /// Create a new strategy.
    /// # Arguments
    /// * `bits` - Size of uint in bits
    pub fn new(bits: usize) -> Self {
        Self { bits, edge_weight: 10usize, random_weight: 50usize }
    }

    fn generate_edge_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
        let rng = runner.rng();
        // Choose if we want values around 0 or max
        let is_min = rng.gen_bool(0.5);
        let offset = rng.gen_range(0..4);
        let start = if is_min { offset } else { self.type_max().saturating_sub(offset) };
        Ok(proptest::num::u128::BinarySearch::new(start))
    }

    fn generate_random_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
        let rng = runner.rng();
        let start = rng.gen_range(0..=self.type_max());

        Ok(proptest::num::u128::BinarySearch::new(start))
    }

    fn type_max(&self) -> u128 {
        if self.bits < 128 {
            (1 << self.bits) - 1
        } else {
            u128::MAX
        }
    }
}

impl Strategy for UintStrategy {
    type Tree = proptest::num::u128::BinarySearch;
    type Value = u128;
    fn new_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
        let total_weight = self.random_weight + self.edge_weight;
        let bias = runner.rng().gen_range(0..total_weight);
        // randomly select one of 2 strategies
        match bias {
            x if x < self.edge_weight => self.generate_edge_tree(runner),
            _ => self.generate_random_tree(runner),
        }
    }
}
