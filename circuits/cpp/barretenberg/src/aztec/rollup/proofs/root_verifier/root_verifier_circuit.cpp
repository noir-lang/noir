#include "./root_verifier_circuit.hpp"

namespace rollup {
namespace proofs {
namespace root_verifier {

using namespace plonk;
using namespace plonk::stdlib::recursion;

recursion_output<outer_curve> root_verifier_circuit(
    OuterComposer& composer,
    root_verifier_tx const& tx,
    std::shared_ptr<waffle::verification_key> const& inner_verification_key)
{
    recursion_output<outer_curve> recursion_output;
    auto recursive_manifest = InnerComposer::create_unrolled_manifest(inner_verification_key->num_public_inputs);
    auto recursive_verification_key = verification_key_pt::from_constants(&composer, inner_verification_key);
    recursion_output = verify_proof<outer_curve, recursive_settings>(&composer,
                                                                     recursive_verification_key,
                                                                     recursive_manifest,
                                                                     waffle::plonk_proof{ tx.proof_data },
                                                                     recursion_output);

    // get the index of the first recursive proof element
    auto idx = (inner_verification_key->recursive_proof_public_input_indices).front();
    for (size_t i = 0; i != idx; i++) {
        composer.set_public_input(recursion_output.public_inputs[i].witness_index);
    }

    recursion_output.add_proof_outputs_as_public_inputs();

    return recursion_output;
}

} // namespace root_verifier
} // namespace proofs
} // namespace rollup
