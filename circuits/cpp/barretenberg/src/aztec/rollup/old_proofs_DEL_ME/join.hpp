#pragma once
#include "rollup_context.hpp"

namespace rollup {
namespace prover {

bool join(rollup_context& ctx,
          uint32_t in_index1,
          uint32_t in_index2,
          tx_note const& in_note1,
          tx_note const& in_note2,
          tx_note const& out_note);

}
} // namespace rollup