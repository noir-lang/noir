use std::collections::HashSet;

use acvm::{AcirField, FieldElement};
use proptest::{
    strategy::{NewTree, Strategy},
    test_runner::TestRunner,
};
use rand::Rng;

type BinarySearch = proptest::num::u128::BinarySearch;

/// Value tree for unsigned ints (up to u128).
/// The strategy combines 2 different strategies, each assigned a specific weight:
/// 1. Generate purely random value in a range. This will first choose bit size uniformly (up `bits`
///    param). Then generate a value for this bit size.
/// 2. Generate a random value around the edges (+/- 3 around 0 and max possible value)
#[derive(Debug)]
pub struct UintStrategy {
    /// Bit size of uint (e.g. 64)
    bits: usize,
    /// A set of fixtures to be generated
    fixtures: Vec<FieldElement>,
    /// The weight for edge cases (+/- 3 around 0 and max possible value)
    edge_weight: usize,
    /// The weight for fixtures
    fixtures_weight: usize,
    /// The weight for purely random values
    random_weight: usize,
}

impl UintStrategy {
    /// Create a new strategy.
    /// # Arguments
    /// * `bits` - Size of uint in bits
    /// * `fixtures` - Set of `FieldElements` representing values which the fuzzer weight towards testing.
    pub fn new(bits: usize, fixtures: &HashSet<FieldElement>) -> Self {
        Self {
            bits,
            // We can only consider the fixtures which fit into the bit width.
            fixtures: fixtures.iter().filter(|f| f.num_bits() <= bits as u32).copied().collect(),
            edge_weight: 10usize,
            fixtures_weight: 40usize,
            random_weight: 50usize,
        }
    }

    /// Generate random numbers starting from near 0 or the maximum of the range.
    fn generate_edge_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
        let rng = runner.rng();
        // Choose if we want values around 0 or max
        let is_min = rng.gen_bool(0.5);
        let offset = rng.gen_range(0..4);
        let start = if is_min { offset } else { self.type_max().saturating_sub(offset) };
        Ok(BinarySearch::new(start))
    }

    /// Pick a random `FieldElement` from the `fixtures` as a starting point for
    /// generating random numbers.
    fn generate_fixtures_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
        // generate random cases if there's no fixtures
        if self.fixtures.is_empty() {
            return self.generate_random_tree(runner);
        }

        // Generate value tree from fixture.
        let fixture = &self.fixtures[runner.rng().gen_range(0..self.fixtures.len())];

        Ok(BinarySearch::new(fixture.to_u128()))
    }

    /// Generate random values between 0 and the MAX with the given bit width.
    fn generate_random_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
        let rng = runner.rng();
        let start = rng.gen_range(0..=self.type_max());

        Ok(BinarySearch::new(start))
    }

    /// Maximum integer that fits in the given bit width.
    fn type_max(&self) -> u128 {
        if self.bits < 128 {
            (1 << self.bits) - 1
        } else {
            u128::MAX
        }
    }
}

impl Strategy for UintStrategy {
    type Tree = BinarySearch;
    type Value = u128;

    /// Pick randomly from the 3 available strategies for generating unsigned integers.
    fn new_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
        let total_weight = self.random_weight + self.fixtures_weight + self.edge_weight;
        let bias = runner.rng().gen_range(0..total_weight);
        // randomly select one of 3 strategies
        match bias {
            x if x < self.edge_weight => self.generate_edge_tree(runner),
            x if x < self.edge_weight + self.fixtures_weight => self.generate_fixtures_tree(runner),
            _ => self.generate_random_tree(runner),
        }
    }
}
