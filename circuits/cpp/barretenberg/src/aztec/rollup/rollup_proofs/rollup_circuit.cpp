#include "rollup_circuit.hpp"

namespace rollup {
namespace rollup_proofs {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::recursion;

std::vector<recursion_output<field_ct, group_ct>> rollup_circuit(
    Composer& composer,
    std::vector<waffle::plonk_proof> const& proofs,
    std::shared_ptr<waffle::verification_key> const& inner_verification_key)
{
    std::vector<recursion_output<field_ct, group_ct>> recursion_outputs(proofs.size());
    for (size_t i = 0; i < proofs.size(); ++i) {
        // TODO: Hardcoding number of public inputs is bad...
        auto recursive_manifest = Composer::create_unrolled_manifest(9);

        auto output = verify_proof<Composer, recursive_turbo_verifier_settings>(
            &composer, inner_verification_key, recursive_manifest, proofs[i]);
        recursion_outputs[i] = output;
    }
    return recursion_outputs;
}

} // namespace rollup_proofs
} // namespace rollup
