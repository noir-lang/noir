#pragma once
#include "types.hpp"

namespace rollup {

bool split(
    rollup_context& ctx, uint32_t in_index, tx_note const& in_note, tx_note const& out_note1, tx_note const& out_note2);


}