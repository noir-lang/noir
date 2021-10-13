#pragma once
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

field_ct compute_nullifier(field_ct const& note_commitment,
                           field_ct const& account_private_key,
                           bool_ct const& is_real_note);

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup