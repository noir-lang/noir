#pragma once
#include "barretenberg/join_split_example/types.hpp"

namespace join_split_example::proofs::notes::circuit {

using namespace bb::plonk::stdlib;

std::pair<bool_ct, suint_ct> deflag_asset_id(suint_ct const& asset_id);

bool_ct get_asset_id_flag(suint_ct const& asset_id);

} // namespace join_split_example::proofs::notes::circuit
