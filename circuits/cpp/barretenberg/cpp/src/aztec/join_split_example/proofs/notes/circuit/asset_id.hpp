#pragma once
#include <stdlib/types/types.hpp>

namespace join_split_example::proofs::notes::circuit {

using namespace plonk::stdlib::types;

std::pair<bool_ct, suint_ct> deflag_asset_id(suint_ct const& asset_id);

bool_ct get_asset_id_flag(suint_ct const& asset_id);

} // namespace join_split_example::proofs::notes::circuit