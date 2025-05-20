use crate::types::hash::accumulate_sha256;

// N = maximum leaves
// For now we only care about the root
pub(crate) struct VariableMerkleTree {
    // leaves: [Field; N],
    pub(crate) root: Field,
}

/// Recursively calculates the smallest power exponent `n` where `2^n` is greater than or equal
/// to the input value, starting from a given exponent.
///
/// # Arguments
///
/// * `input` - Target value to find the next power of 2 that exceeds or equals it
/// * `start` - Initial exponent to start checking from
///
/// # Returns
///
/// The smallest exponent `n` where `2^n >= input`
unconstrained fn get_next_power_exponent(input: u32, start: u8) -> u8 {
    let mut next_power_exponent = 0;
    // We check if input is less than or equal to 2^start.
    if input <= (1 << start) {
        next_power_exponent = start;
    } else {
        next_power_exponent = get_next_power_exponent(input, start + 1);
    }
    next_power_exponent
}

/// Calculates the previous power of 2.
///
/// The previous power of 2 is the largest power of 2 that is smaller than `value`.
/// For example:
/// - For 7, previous power of 2 is 4 (2^2)
/// - For 10, previous power of 2 is 8 (2^3)
/// - For 16, previous power of 2 is 8 (2^3)
///
/// # Arguments
/// * value - Integer value for which we need to find the previous power of 2
///
/// # Returns
/// The previous power of 2
fn get_prev_power_2(value: u32) -> u32 {
    // Safety: This is a hint that we'll use to compute the next and previous powers of two, which we check to be
    // larger and smaller than respectively. The get_next_power_exponent function happens to return exactly what
    // we need.
    let next_power_exponent = unsafe { get_next_power_exponent(value, 0) };

    let next_power_2 = 1 << next_power_exponent;
    let prev_power_2 = next_power_2 / 2;
    assert(prev_power_2 < value);
    assert(value <= next_power_2);

    prev_power_2
}

// Calculates the root of the minimal size merkle tree required to store num_non_empty_leaves.
// Since we cannot isolate branches, it doesn't cost fewer gates than using MerkleTree on the full array of elements N,
// but is slightly cheaper on-chain and cleaner elsewhere.
impl VariableMerkleTree {
    // Example - tx_0 with 3 msgs | tx_1 with 2 msgs creates:
    //
    //        tx0oh           tx1oh
    //     /          \      /    \
    //   msg01     msg23    msg3 msg4
    //  /     \    /    \
    // msg0 msg1 msg2    0
    // |       tx_0      |  | tx_1 |
    //
    pub(crate) fn new_sha<let N: u32>(leaves: [Field; N], num_non_empty_leaves: u32) -> Self {
        let num_nodes_layer_1 = if (num_non_empty_leaves == 0) {
            // For 0 leaves, there is no layer 1, no hashing happens and the root is set to 0
            0
        } else if (num_non_empty_leaves == 1) {
            // For 1 leaf, 1 round of hashing happens and root = hash([leaf, 0])
            1
        } else {
            // For more than 1 leaf, we dynamically compute num of nodes in layer 1 by finding the previous power of 2
            get_prev_power_2(num_non_empty_leaves)
        };

        // We hash the base layer
        let mut nodes = [0; N];
        for i in 0..N / 2 {
            if (i < num_nodes_layer_1) {
                nodes[i] = accumulate_sha256([leaves[2 * i], leaves[2 * i + 1]]);
            }
        }

        // We hash the other layers
        let mut stop = num_non_empty_leaves < 3;

        let mut next_layer_end = num_nodes_layer_1 / 2;
        let mut next_layer_size = next_layer_end;
        let mut root = nodes[0];
        for i in 0..(N - 1 - N / 2) {
            if !stop {
                nodes[num_nodes_layer_1 + i] = accumulate_sha256([nodes[2 * i], nodes[2 * i + 1]]);
                if i == next_layer_end {
                    // Reached next layer => move up one layer
                    next_layer_size = next_layer_size / 2;
                    next_layer_end += next_layer_size;
                }
                if (next_layer_size == 1) {
                    // Reached root
                    root = nodes[num_nodes_layer_1 + i];
                    stop = true;
                }
            }
        }

        VariableMerkleTree { root }
    }

    pub(crate) fn get_root(self) -> Field {
        self.root
    }
}
