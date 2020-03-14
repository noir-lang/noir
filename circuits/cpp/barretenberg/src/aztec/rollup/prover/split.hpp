#pragma once
#include "../tx/tx_note.hpp"
#include "rollup_context.hpp"

namespace rollup {
namespace prover {

using namespace rollup::tx;

bool split(
    rollup_context& ctx, uint32_t in_index, tx_note const& in_note, tx_note const& out_note1, tx_note const& out_note2);

} // namespace prover
} // namespace rollup