#pragma once
#include "barretenberg/examples/join_split/types.hpp"

namespace bb::join_split_example::proofs::notes::circuit {

using namespace bb::stdlib;

std::pair<bool_ct, suint_ct> deflag_asset_id(suint_ct const& asset_id);

bool_ct get_asset_id_flag(suint_ct const& asset_id);

} // namespace bb::join_split_example::proofs::notes::circuit
