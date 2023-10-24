#pragma once
#include "barretenberg/join_split_example/types.hpp"

namespace join_split_example::proofs::notes::circuit {

field_ct compute_nullifier(field_ct const& note_commitment,
                           field_ct const& account_private_key,
                           bool_ct const& is_note_in_use);

} // namespace join_split_example::proofs::notes::circuit
