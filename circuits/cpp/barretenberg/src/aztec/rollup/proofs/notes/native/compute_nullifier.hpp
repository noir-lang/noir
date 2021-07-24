#pragma once
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

barretenberg::fr compute_nullifier(grumpkin::fq const& note_commitment,
                                   const uint32_t tree_index,
                                   grumpkin::fr const& account_private_key,
                                   const bool is_real_note);

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup