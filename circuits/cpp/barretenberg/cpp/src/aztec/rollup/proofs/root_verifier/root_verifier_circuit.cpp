#include "./root_verifier_circuit.hpp"

namespace rollup {
namespace proofs {
namespace root_verifier {

using namespace plonk;
using namespace plonk::stdlib::recursion;

recursion_output<outer_curve> root_verifier_circuit(
    OuterComposer& composer,
    root_verifier_tx const& tx,
    std::shared_ptr<waffle::verification_key> const& inner_verification_key,
    std::vector<std::shared_ptr<waffle::verification_key>> const& valid_vks)
{
    recursion_output<outer_curve> recursion_output;
    if (!valid_vks.size()) {
        composer.failure("Cannot build root verifier circuit with empty list of keys.");
        return recursion_output;
    }

    auto recursive_manifest = InnerComposer::create_unrolled_manifest(inner_verification_key->num_public_inputs);
    auto recursive_verification_key = verification_key_pt::from_witness(&composer, inner_verification_key);
    recursive_verification_key->validate_key_is_in_set(valid_vks);
    recursion_output = verify_proof<outer_curve, recursive_settings>(&composer,
                                                                     recursive_verification_key,
                                                                     recursive_manifest,
                                                                     waffle::plonk_proof{ tx.proof_data },
                                                                     recursion_output);

    // Expose the broadcast data hash, and recursion point inputs.
    recursion_output.public_inputs[0].set_public();
    recursion_output.add_proof_outputs_as_public_inputs();

    return recursion_output;
}

} // namespace root_verifier
} // namespace proofs
} // namespace rollup
