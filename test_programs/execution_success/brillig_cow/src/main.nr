// Tests the copy on write optimization for arrays. We look for cases where we are modifying an array in place when we shouldn't.
global ARRAY_SIZE: u32 = 5;

struct ExecutionResult {
    original: [Field; ARRAY_SIZE],
    modified_once: [Field; ARRAY_SIZE],
    modified_twice: [Field; ARRAY_SIZE],
}

impl ExecutionResult {
    fn is_equal(self, other: ExecutionResult) -> bool {
        (self.original == other.original)
            & (self.modified_once == other.modified_once)
            & (self.modified_twice == other.modified_twice)
    }
}

fn modify_in_inlined_constrained(original: [Field; ARRAY_SIZE], index: u64) -> ExecutionResult {
    let mut modified = original;

    modified[index] = 27;

    let modified_once = modified;

    modified[index + 1] = 27;

    ExecutionResult { original, modified_once, modified_twice: modified }
}

unconstrained fn modify_in_unconstrained(
    original: [Field; ARRAY_SIZE],
    index: u64,
) -> ExecutionResult {
    let mut modified = original;

    modified[index] = 27;

    let modified_once = modified;

    modified[index + 1] = 27;

    ExecutionResult { original, modified_once, modified_twice: modified }
}

unconstrained fn main(original: [Field; ARRAY_SIZE], index: u64, expected_result: ExecutionResult) {
    assert(expected_result.is_equal(modify_in_unconstrained(original, index)));
    assert(expected_result.is_equal(modify_in_inlined_constrained(original, index)));
}
