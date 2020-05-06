#pragma once
#include <stdlib/recursion/verifier/verifier.hpp>
#include <stdlib/recursion/verifier/program_settings.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace rollup_proofs {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;

std::vector<recursion_output<field_ct, group_ct>> rollup_circuit(
    Composer& composer,
    std::vector<waffle::plonk_proof> const& proofs,
    std::shared_ptr<waffle::verification_key> const& inner_verification_key);

} // namespace rollup_proofs
} // namespace rollup
