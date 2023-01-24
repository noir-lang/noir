#pragma once
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

barretenberg::fr compute_nullifier(grumpkin::fq const& note_commitment,
                                   grumpkin::fr const& account_private_key,
                                   const bool is_note_in_use);

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup