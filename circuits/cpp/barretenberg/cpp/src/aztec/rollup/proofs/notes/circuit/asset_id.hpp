#pragma once
#include <stdlib/types/turbo.hpp>

namespace rollup::proofs::notes::circuit {

using namespace plonk::stdlib::types::turbo;

std::pair<bool_ct, suint_ct> deflag_asset_id(suint_ct const& asset_id);

bool_ct get_asset_id_flag(suint_ct const& asset_id);

} // namespace rollup::proofs::notes::circuit