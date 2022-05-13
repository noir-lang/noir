#include <stdlib/types/turbo.hpp>
#include "../constants.hpp"

namespace rollup::proofs::notes::circuit {

using namespace plonk::stdlib::types::turbo;

std::pair<bool_ct, suint_ct> deflag_asset_id(suint_ct const& asset_id)
{
    const auto virtual_idx = MAX_NUM_ASSETS_BIT_LENGTH - 1; //  bit 29

    // Extract the most significant bit of the asset id: bit 29-30.
    const auto sliced_asset_id = asset_id.slice(virtual_idx + 1, virtual_idx);

    // 'virtual' notes defined by asset id msb being +1.
    // A virtual note does not have an ERC20 token equivalent and exists only inside the Aztec network.
    // The low 29 bits of the asset id represent the defi interaction nonce of the defi interaction that created the
    // note.
    const bool_ct is_virtual = sliced_asset_id[1] == 1;
    const suint_ct& deflagged_asset_id = sliced_asset_id[0];

    return { is_virtual, deflagged_asset_id };
}

bool_ct get_asset_id_flag(suint_ct const& asset_id)
{
    const auto& [is_virtual, _] = deflag_asset_id(asset_id);
    return is_virtual;
}

} // namespace rollup::proofs::notes::circuit