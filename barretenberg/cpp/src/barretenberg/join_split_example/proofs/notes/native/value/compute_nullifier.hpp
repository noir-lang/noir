#pragma once
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace bb::join_split_example::proofs::notes::native {

bb::fr compute_nullifier(grumpkin::fq const& note_commitment,
                         grumpkin::fr const& account_private_key,
                         const bool is_note_in_use);

} // namespace bb::join_split_example::proofs::notes::native
