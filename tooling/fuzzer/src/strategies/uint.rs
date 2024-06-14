use proptest::{
    strategy::{NewTree, Strategy, ValueTree},
    test_runner::TestRunner,
};
use rand::Rng;

/// Value tree for unsigned ints (up to uint256).
pub struct UintValueTree {
    /// Lower base
    lo: u128,
    /// Current value
    curr: u128,
    /// Higher base
    hi: u128,
    /// If true cannot be simplified or complexified
    fixed: bool,
}

impl UintValueTree {
    /// Create a new tree
    /// # Arguments
    /// * `start` - Starting value for the tree
    /// * `fixed` - If `true` the tree would only contain one element and won't be simplified.
    fn new(start: u128, fixed: bool) -> Self {
        Self { lo: 0, curr: start, hi: start, fixed }
    }

    fn reposition(&mut self) -> bool {
        let interval = self.hi - self.lo;
        let new_mid = self.lo + interval / 2;

        if new_mid == self.curr {
            false
        } else {
            self.curr = new_mid;
            true
        }
    }
}

impl ValueTree for UintValueTree {
    type Value = u128;

    fn current(&self) -> Self::Value {
        self.curr
    }

    fn simplify(&mut self) -> bool {
        if self.fixed || (self.hi <= self.lo) {
            return false;
        }
        self.hi = self.curr;
        self.reposition()
    }

    fn complicate(&mut self) -> bool {
        if self.fixed || (self.hi <= self.lo) {
            return false;
        }

        self.lo = self.curr.wrapping_add(1);
        self.reposition()
    }
}

/// Value tree for unsigned ints (up to uint256).
/// The strategy combines 3 different strategies, each assigned a specific weight:
/// 1. Generate purely random value in a range. This will first choose bit size uniformly (up `bits`
///    param). Then generate a value for this bit size.
/// 2. Generate a random value around the edges (+/- 3 around 0 and max possible value)
/// 3. Generate a value from a predefined fixtures set
///
/// To define uint fixtures:
/// - return an array of possible values for a parameter named `amount` declare a function `function
///   fixture_amount() public returns (uint32[] memory)`.
/// - use `amount` named parameter in fuzzed test in order to include fixtures in fuzzed values
///   `function testFuzz_uint32(uint32 amount)`.
///
/// If fixture is not a valid uint type then error is raised and random value generated.
#[derive(Debug)]
pub struct UintStrategy {
    /// Bit size of uint (e.g. 256)
    bits: usize,

    /// The weight for edge cases (+/- 3 around 0 and max possible value)
    edge_weight: usize,
    /// The weight for purely random values
    random_weight: usize,
}

impl UintStrategy {
    /// Create a new strategy.
    /// #Arguments
    /// * `bits` - Size of uint in bits
    /// * `fixtures` - A set of fixed values to be generated (according to fixtures weight)
    pub fn new(bits: usize) -> Self {
        Self { bits, edge_weight: 10usize, random_weight: 50usize }
    }

    fn generate_edge_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
        let rng = runner.rng();
        // Choose if we want values around 0 or max
        let is_min = rng.gen_bool(0.5);
        let offset = rng.gen_range(0..4);
        let start = if is_min { offset } else { self.type_max().saturating_sub(offset) };
        Ok(UintValueTree::new(start, false))
    }

    fn generate_random_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
        let rng = runner.rng();
        let start: u128 = rng.gen_range(0..=self.type_max());

        Ok(UintValueTree::new(start, false))
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
    type Tree = UintValueTree;
    type Value = u128;
    fn new_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
        let total_weight = self.random_weight + self.edge_weight;
        let bias = runner.rng().gen_range(0..total_weight);
        // randomly select one of 3 strategies
        match bias {
            x if x < self.edge_weight => self.generate_edge_tree(runner),
            _ => self.generate_random_tree(runner),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::strategies::uint::UintValueTree;
    use proptest::strategy::ValueTree;

    #[test]
    fn test_uint_tree_complicate_max() {
        let mut uint_tree = UintValueTree::new(u128::MAX, false);
        assert_eq!(uint_tree.hi, u128::MAX);
        assert_eq!(uint_tree.curr, u128::MAX);
        uint_tree.complicate();
        assert_eq!(uint_tree.lo, u128::MIN);
    }
}
