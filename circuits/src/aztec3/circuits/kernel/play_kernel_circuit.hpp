#include <stdlib/types/turbo.hpp>
#include <stdlib/recursion/verifier/verifier.hpp>
#include <stdlib/recursion/verifier/program_settings.hpp>
#include <plonk/transcript/manifest.hpp>

namespace aztec3::circuits::kernel {

using namespace plonk::stdlib::types::turbo;
using plonk::stdlib::recursion::recursion_output;
using plonk::stdlib::recursion::recursive_turbo_verifier_settings;
using plonk::stdlib::recursion::verification_key;
using plonk::stdlib::recursion::verify_proof;
using transcript::Manifest;

class TurboRecursion {
  public:
    using AggregationObject = recursion_output<bn254>;
    using Proof = waffle::plonk_proof;
    using VK = std::shared_ptr<waffle::verification_key>;
    static AggregationObject aggregate(Composer* composer,
                                       const VK& vk,
                                       const Proof& proof,
                                       const AggregationObject recursion_output = AggregationObject())
    {
        std::shared_ptr<verification_key<bn254>> recursive_vk = verification_key<bn254>::from_witness(composer, vk);
        const transcript::Manifest recursive_manifest = Composer::create_unrolled_manifest(vk->num_public_inputs);

        AggregationObject result = verify_proof<bn254, recursive_turbo_verifier_settings<bn254>>(
            composer, recursive_vk, recursive_manifest, proof, recursion_output);

        return result;
    }
};

template <class Recursion>
typename Recursion::AggregationObject play_kernel_circuit(Composer& composer,
                                                          typename Recursion::VK const& app_vk,
                                                          typename Recursion::Proof const& app_proof)
{
    typename Recursion::AggregationObject recursion_output = Recursion::aggregate(&composer, app_vk, app_proof);

    recursion_output.add_proof_outputs_as_public_inputs();

    return recursion_output;
};

// TODO: build an app first.
template <class Recursion>
typename Recursion::AggregationObject play_kernel_circuit_2(Composer& composer,
                                                            typename Recursion::VK const& app_vk,
                                                            typename Recursion::Proof const& app_proof,
                                                            typename Recursion::VK const& prev_kernel_vk,
                                                            typename Recursion::Proof const& prev_kernel_proof)
{
    typename Recursion::AggregationObject recursion_output =
        Recursion::aggregate(&composer, prev_kernel_vk, prev_kernel_proof);

    recursion_output = Recursion::aggregate(&composer, app_vk, app_proof, recursion_output);

    recursion_output.add_proof_outputs_as_public_inputs();

    return recursion_output;
};

// circuit_data get_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs, bool mock)
// {
//     std::cerr << "Getting join-split circuit data..." << std::endl;

//     auto build_circuit = [&](Composer& composer) {
//         join_split_tx tx(noop_tx());
//         join_split_circuit(composer, tx);
//     };

//     return proofs::get_circuit_data<Composer>(
//         "join split", "", srs, "", true, false, false, true, true, true, mock, build_circuit);
// }

// recursion_output<bn254> play_kernel_circuit(Composer& composer,
//                                             std::shared_ptr<waffle::verification_key> const& app_vk,
//                                             waffle::plonk_proof const& app_proof)
// {
//     const transcript::Manifest recursive_manifest = Composer::create_unrolled_manifest(app_vk->num_public_inputs);

//     std::shared_ptr<verification_key<bn254>> recursive_vk = verification_key<bn254>::from_witness(&composer, app_vk);

//     recursion_output<bn254> recursion_output = verify_proof<bn254, recursive_turbo_verifier_settings<bn254>>(
//         &composer, recursive_vk, recursive_manifest, app_proof);

//     // recursion_output<bn254> recursion_output;

//     // recursion_output = verify_proof<bn254, recursive_turbo_verifier_settings<bn254>>(
//     //     &composer, recursive_vk, recursive_manifest, app_proof, recursion_output);

//     recursion_output.add_proof_outputs_as_public_inputs();

//     return recursion_output;
// };

} // namespace aztec3::circuits::kernel