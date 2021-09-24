#pragma once
#include "./root_verifier_tx.hpp"
#include <stdlib/recursion/verifier/program_settings.hpp>
#include <stdlib/recursion/verifier/verifier.hpp>

namespace rollup {
namespace proofs {
namespace root_verifier {

using namespace plonk;

using InnerComposer = waffle::TurboComposer;
using OuterComposer = waffle::StandardComposer;

typedef stdlib::bn254<OuterComposer> outer_curve;

typedef stdlib::recursion::verification_key<outer_curve> verification_key_pt;
typedef stdlib::recursion::recursive_turbo_verifier_settings<outer_curve> recursive_settings;

struct circuit_outputs {
    stdlib::recursion::recursion_output<outer_curve> recursion_output;
    std::shared_ptr<verification_key_pt> verification_key;
};

stdlib::recursion::recursion_output<outer_curve> root_verifier_circuit(
    OuterComposer& composer,
    root_verifier_tx const& tx,
    std::shared_ptr<waffle::verification_key> const& inner_verification_key);

} // namespace root_verifier
} // namespace proofs
} // namespace rollup
