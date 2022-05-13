#include "../constants.hpp"

namespace rollup::proofs::notes::native {

std::pair<bool, uint32_t> deflag_asset_id(uint32_t const& asset_id)
{
    const bool is_virtual = (asset_id >> (MAX_NUM_ASSETS_BIT_LENGTH - 1)) == 1;

    const uint32_t deflagged_asset_id =
        is_virtual ? asset_id - (uint32_t(1) << (MAX_NUM_ASSETS_BIT_LENGTH - 1)) : asset_id;

    return { is_virtual, deflagged_asset_id };
}

bool get_asset_id_flag(uint32_t const& asset_id)
{
    const auto& [is_virtual, _] = deflag_asset_id(asset_id);
    return is_virtual;
}

} // namespace rollup::proofs::notes::native