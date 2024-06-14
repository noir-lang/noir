use proptest::{
    strategy::{NewTree, Strategy, ValueTree},
    test_runner::TestRunner,
};
use rand::Rng;

/// Value tree for signed ints (up to int256).
pub struct IntValueTree {
    /// Lower base (by absolute value)
    lo: i128,
    /// Current value
    curr: i128,
    /// Higher base (by absolute value)
    hi: i128,
    /// If true cannot be simplified or complexified
    fixed: bool,
}

impl IntValueTree {
    /// Create a new tree
    /// # Arguments
    /// * `start` - Starting value for the tree
    /// * `fixed` - If `true` the tree would only contain one element and won't be simplified.
    fn new(start: i128, fixed: bool) -> Self {
        Self { lo: 0, curr: start, hi: start, fixed }
    }

    fn reposition(&mut self) -> bool {
        let interval = self.hi - self.lo;
        let new_mid = self.lo + interval / 2i128;

        if new_mid == self.curr {
            false
        } else {
            self.curr = new_mid;
            true
        }
    }

    fn magnitude_greater(lhs: i128, rhs: i128) -> bool {
        if lhs == 0 {
            return false;
        }
        (lhs > rhs) ^ (lhs.is_negative())
    }
}

impl ValueTree for IntValueTree {
    type Value = i128;

    fn current(&self) -> Self::Value {
        self.curr
    }

    fn simplify(&mut self) -> bool {
        if self.fixed || !Self::magnitude_greater(self.hi, self.lo) {
            return false;
        }
        self.hi = self.curr;
        self.reposition()
    }

    fn complicate(&mut self) -> bool {
        if self.fixed || !Self::magnitude_greater(self.hi, self.lo) {
            return false;
        }

        self.lo = if self.curr != i128::MIN && self.curr != i128::MAX {
            self.curr + if self.hi.is_negative() { -1i128 } else { 1i128 }
        } else {
            self.curr
        };

        self.reposition()
    }
}

/// Value tree for signed ints (up to int256).
/// The strategy combines 3 different strategies, each assigned a specific weight:
/// 1. Generate purely random value in a range. This will first choose bit size uniformly (up `bits`
///    param). Then generate a value for this bit size.
/// 2. Generate a random value around the edges (+/- 3 around min, 0 and max possible value)
/// 3. Generate a value from a predefined fixtures set
///
/// To define int fixtures:
/// - return an array of possible values for a parameter named `amount` declare a function `function
///   fixture_amount() public returns (int32[] memory)`.
/// - use `amount` named parameter in fuzzed test in order to include fixtures in fuzzed values
///   `function testFuzz_int32(int32 amount)`.
///
/// If fixture is not a valid int type then error is raised and random value generated.
#[derive(Debug)]
pub struct IntStrategy {
    /// Bit size of int (e.g. 256)
    bits: usize,
    /// The weight for edge cases (+/- 3 around 0 and max possible value)
    edge_weight: usize,
    /// The weight for purely random values
    random_weight: usize,
}

impl IntStrategy {
    /// Create a new strategy.
    /// #Arguments
    /// * `bits` - Size of uint in bits
    /// * `fixtures` - A set of fixed values to be generated (according to fixtures weight)
    pub fn new(bits: usize) -> Self {
        Self { bits, edge_weight: 10usize, random_weight: 50usize }
    }

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
        Ok(IntValueTree::new(start, false))
    }

    fn generate_random_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
        let rng = runner.rng();

        let start: i128 = rng.gen_range(self.type_min()..=self.type_max());
        Ok(IntValueTree::new(start, false))
    }

    fn type_max(&self) -> i128 {
        if self.bits < 128 {
            (1i128 << (self.bits - 1)) - 1
        } else {
            i128::MAX
        }
    }

    fn type_min(&self) -> i128 {
        if self.bits < 128 {
            -(1i128 << (self.bits - 1))
        } else {
            i128::MIN
        }
    }
}

impl Strategy for IntStrategy {
    type Tree = IntValueTree;
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

#[cfg(test)]
mod tests {
    use crate::strategies::int::IntValueTree;
    use proptest::strategy::ValueTree;

    #[test]
    fn test_int_tree_complicate_should_not_overflow() {
        let mut int_tree = IntValueTree::new(i128::MAX, false);
        assert_eq!(int_tree.hi, i128::MAX);
        assert_eq!(int_tree.curr, i128::MAX);
        int_tree.complicate();
        assert_eq!(int_tree.lo, i128::MAX);

        let mut int_tree = IntValueTree::new(i128::MIN, false);
        assert_eq!(int_tree.hi, i128::MIN);
        assert_eq!(int_tree.curr, i128::MIN);
        int_tree.complicate();
        assert_eq!(int_tree.lo, i128::MIN);
    }
}
