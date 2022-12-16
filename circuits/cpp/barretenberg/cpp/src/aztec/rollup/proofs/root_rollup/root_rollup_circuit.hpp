#pragma once
#include "./root_rollup_tx.hpp"
#include <stdlib/recursion/verifier/program_settings.hpp>
#include <stdlib/recursion/verifier/verifier.hpp>
#include <stdlib/types/types.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace plonk::stdlib::types;
using namespace plonk::stdlib::recursion;

struct circuit_result_data {
    plonk::stdlib::recursion::recursion_output<bn254> recursion_output;
    std::vector<fr> broadcast_data;
};

circuit_result_data root_rollup_circuit(Composer& composer,
                                        root_rollup_tx const& rollups,
                                        size_t inner_rollup_size,
                                        size_t outer_rollup_size,
                                        std::shared_ptr<waffle::verification_key> const& inner_verification_key);

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
