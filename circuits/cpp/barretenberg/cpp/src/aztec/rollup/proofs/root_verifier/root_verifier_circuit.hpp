#pragma once
#include "./root_verifier_tx.hpp"
#include <stdlib/recursion/verifier/program_settings.hpp>
#include <stdlib/recursion/verifier/verifier.hpp>
#include <stdlib/types/types.hpp>

namespace rollup {
namespace proofs {
namespace root_verifier {

using namespace plonk;

/**
 * The InnerComposer is the composer used by the inner circuits, i.e. the rollup and root rollup circuits.
 * Note that the InnerComposer can only be set to Turbo or Ultra. Although it can also work with Standard composer
 * theoretically, the size of the rollup and root rollup circuits with standard composer would be humongous.
 * As such, we don't need the InnerComposer to be standard in any case for efficiency reasons.
 */
typedef std::
    conditional_t<stdlib::types::SYSTEM_COMPOSER == waffle::PLOOKUP, waffle::UltraComposer, waffle::TurboComposer>
        InnerComposer;
using OuterComposer = waffle::StandardComposer;

typedef stdlib::bn254<OuterComposer> outer_curve;

typedef stdlib::recursion::verification_key<outer_curve> verification_key_pt;
typedef std::conditional_t<stdlib::types::SYSTEM_COMPOSER == waffle::PLOOKUP,
                           stdlib::recursion::recursive_ultra_to_standard_verifier_settings<outer_curve>,
                           stdlib::recursion::recursive_turbo_verifier_settings<outer_curve>>
    recursive_settings;

struct circuit_outputs {
    stdlib::recursion::recursion_output<outer_curve> recursion_output;
    std::shared_ptr<verification_key_pt> verification_key;
};

stdlib::recursion::recursion_output<outer_curve> root_verifier_circuit(
    OuterComposer& composer,
    root_verifier_tx const& tx,
    std::shared_ptr<waffle::verification_key> const& inner_verification_key,
    std::vector<std::shared_ptr<waffle::verification_key>> const& valid_vks);

} // namespace root_verifier
} // namespace proofs
} // namespace rollup
