#pragma once
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace native {

bb::fr compute_nullifier(grumpkin::fq const& note_commitment,
                         grumpkin::fr const& account_private_key,
                         const bool is_note_in_use);

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace join_split_example
