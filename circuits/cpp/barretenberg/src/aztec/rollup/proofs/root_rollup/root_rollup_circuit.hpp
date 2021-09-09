#pragma once
#include "./root_rollup_tx.hpp"
#include <stdlib/recursion/verifier/program_settings.hpp>
#include <stdlib/recursion/verifier/verifier.hpp>
#include <stdlib/types/turbo.hpp>
#include "./root_rollup_proof_data.hpp"
namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;

recursion_output<bn254> root_rollup_circuit(Composer& composer,
                                            root_rollup_tx const& rollups,
                                            size_t inner_rollup_size,
                                            size_t outer_rollup_size,
                                            std::shared_ptr<waffle::verification_key> const& inner_verification_key);

recursion_output<bn254> root_rollup_circuit(Composer& composer,
                                            root_rollup_tx const& rollups,
                                            size_t inner_rollup_size,
                                            size_t outer_rollup_size,
                                            std::shared_ptr<waffle::verification_key> const& inner_verification_key,
                                            root_rollup_proof_data& rollup_data);

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
