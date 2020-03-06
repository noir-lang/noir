#pragma once
#include <barretenberg/misc_crypto/commitment/pedersen_note.hpp>
#include <barretenberg/misc_crypto/schnorr/schnorr.hpp>
#include "types.hpp"
#include "join_split_tx.hpp"

namespace rollup {

bool join_split(rollup_context& ctx, join_split_tx const& tx);

} // namespace rollup