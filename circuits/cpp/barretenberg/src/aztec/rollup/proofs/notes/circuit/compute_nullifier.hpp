#pragma once
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

field_ct compute_nullifier(point_ct const& encrypted_note,
                           field_ct const& tree_index,
                           field_ct const& account_private_key,
                           bool_ct const& is_real_note);

} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup