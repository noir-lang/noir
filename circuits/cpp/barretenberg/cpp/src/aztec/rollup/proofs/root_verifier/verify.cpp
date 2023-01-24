#include "./verify.hpp"
#include "./root_verifier_circuit.hpp"

namespace rollup {
namespace proofs {
namespace root_verifier {

verify_result<OuterComposer> build_circuit(OuterComposer& composer,
                                           root_verifier_tx& tx,
                                           circuit_data const& cd,
                                           root_rollup::circuit_data const& root_rollup_cd)
{
    verify_result<OuterComposer> result;

    if (!root_rollup_cd.verification_key) {
        info("Inner verification key not provided.");
        return result;
    }

    result.recursion_output = root_verifier_circuit(composer, tx, root_rollup_cd.verification_key, cd.valid_vks);
    return result;
}

verify_result<OuterComposer> verify_logic(root_verifier_tx& tx,
                                          circuit_data const& cd,
                                          root_rollup::circuit_data const& root_rollup_cd)
{
    OuterComposer composer = OuterComposer(cd.proving_key, cd.verification_key, cd.num_gates);
    return verify_logic_internal(
        composer, tx, cd, "root verifier", [&](OuterComposer& composer, root_verifier_tx& tx, circuit_data const& cd) {
            return build_circuit(composer, tx, cd, root_rollup_cd);
        });
}

verify_result<OuterComposer> verify(root_verifier_tx& tx,
                                    circuit_data const& cd,
                                    root_rollup::circuit_data const& root_rollup_cd)
{
    OuterComposer composer = OuterComposer(cd.proving_key, cd.verification_key, cd.num_gates);
    return verify_internal(composer,
                           tx,
                           cd,
                           "root verifier",
                           false,
                           [&](OuterComposer& composer, root_verifier_tx& tx, circuit_data const& cd) {
                               return build_circuit(composer, tx, cd, root_rollup_cd);
                           });
}

} // namespace root_verifier
} // namespace proofs
} // namespace rollup