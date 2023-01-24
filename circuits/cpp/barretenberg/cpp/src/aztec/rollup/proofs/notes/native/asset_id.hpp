#pragma once
#include <utility>
#include <stdint.h>

namespace rollup::proofs::notes::native {

std::pair<bool, uint32_t> deflag_asset_id(uint32_t const& asset_id);

bool get_asset_id_flag(uint32_t const& asset_id);

} // namespace rollup::proofs::notes::native