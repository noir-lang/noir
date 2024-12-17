use proptest::{
    strategy::{NewTree, Strategy},
    test_runner::TestRunner,
};
use rand::Rng;

type BinarySearch = proptest::num::i128::BinarySearch;

/// Strategy for signed ints (up to i128).
/// The strategy combines 2 different strategies, each assigned a specific weight:
/// 1. Generate purely random value in a range. This will first choose bit size uniformly (up `bits`
///    param). Then generate a value for this bit size.
/// 2. Generate a random value around the edges (+/- 3 around min, 0 and max possible value)
#[derive(Debug)]
pub struct IntStrategy {
    /// Bit size of int (e.g. 128)
    bits: usize,
    /// The weight for edge cases (+/- 3 around 0 and max possible value)
    edge_weight: usize,
    /// The weight for purely random values
    random_weight: usize,
}

impl IntStrategy {
    /// Create a new strategy.
    /// # Arguments
    /// * `bits` - Size of int in bits
    pub fn new(bits: usize) -> Self {
        Self { bits, edge_weight: 10usize, random_weight: 50usize }
    }

    /// Generate random values near MIN or the MAX value.
    fn generate_edge_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
        let rng = runner.rng();

        let offset = rng.gen_range(0..4);
        // Choose if we want values around min, -0, +0, or max
        let kind = rng.gen_range(0..4);
        let start = match kind {
            0 => self.type_min() + offset,
            1 => -offset - 1i128,
            2 => offset,
            3 => self.type_max() - offset,
            _ => unreachable!(),
        };
        Ok(BinarySearch::new(start))
    }

    fn generate_random_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
        let rng = runner.rng();
        let start: i128 = rng.gen_range(self.type_min()..=self.type_max());
        Ok(BinarySearch::new(start))
    }

    /// Maximum allowed positive number.
    fn type_max(&self) -> i128 {
        if self.bits < 128 {
            (1i128 << (self.bits - 1)) - 1
        } else {
            i128::MAX
        }
    }

    /// Minimum allowed negative number.
    fn type_min(&self) -> i128 {
        if self.bits < 128 {
            -(1i128 << (self.bits - 1))
        } else {
            i128::MIN
        }
    }
}

impl Strategy for IntStrategy {
    type Tree = BinarySearch;
    type Value = i128;

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
