#pragma once
#include "rollup_context.hpp"
#include "../tx/join_split_tx.hpp"

namespace rollup {
namespace prover {

using namespace rollup::tx;

bool join_split(rollup_context& ctx, join_split_tx const& tx);

} // namespace rollup
}