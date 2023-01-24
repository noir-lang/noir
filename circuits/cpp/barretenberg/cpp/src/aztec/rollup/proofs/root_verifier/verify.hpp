#pragma once
#include "../verify.hpp"
#include "compute_circuit_data.hpp"
#include "../root_rollup/index.hpp"

namespace rollup {
namespace proofs {
namespace root_verifier {

verify_result<OuterComposer> verify_logic(root_verifier_tx& tx,
                                          circuit_data const& circuit_data,
                                          root_rollup::circuit_data const& root_rollup_cd);

verify_result<OuterComposer> verify(root_verifier_tx& tx,
                                    circuit_data const& circuit_data,
                                    root_rollup::circuit_data const& root_rollup_cd);

} // namespace root_verifier
} // namespace proofs
} // namespace rollup
