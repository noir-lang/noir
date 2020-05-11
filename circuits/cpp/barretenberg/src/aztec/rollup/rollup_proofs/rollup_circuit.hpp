#pragma once
#include "rollup_tx.hpp"
#include <stdlib/recursion/verifier/verifier.hpp>
#include <stdlib/recursion/verifier/program_settings.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace rollup_proofs {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;

std::vector<recursion_output<field_ct, group_ct>> rollup_circuit(
    Composer& composer,
    rollup_tx const& proofs,
    std::shared_ptr<waffle::verification_key> const& inner_verification_key,
    size_t rollup_size);

} // namespace rollup_proofs
} // namespace rollup
