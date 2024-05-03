use crate::{abis::max_block_number::MaxBlockNumber, traits::Empty};

// These are validation requests that cannot be fulfilled in the current context (private or public), and must be 
// instead forwarded to the rollup for it to take care of them.
struct RollupValidationRequests {
    max_block_number: MaxBlockNumber,
}

impl Empty for RollupValidationRequests {
    fn empty() -> Self {
        RollupValidationRequests {
            max_block_number: MaxBlockNumber::empty(),
        }
    }
}

