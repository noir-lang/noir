#include <stdlib/types/types.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/recursion/verifier/verifier.hpp>
#include <stdlib/recursion/verifier/program_settings.hpp>
#include <transcript/manifest.hpp>

namespace aztec3::circuits::recursion {

// These are all Circuit Types!
// using plonk::stdlib::recursion::recursion_output;
using plonk::stdlib::types::recursive_inner_verifier_settings;
// using plonk::stdlib::recursion::verification_key;
using plonk::stdlib::recursion::verify_proof;
// using plonk::stdlib::types::bn254;
using plonk::stdlib::types::Composer;
using CT = plonk::stdlib::types::CircuitTypes<Composer>;
using NT = plonk::stdlib::types::NativeTypes;
using transcript::Manifest;

// class Aggregator {
//   public:
//     // Circuit Types:
//     using AggregationObject = recursion_output<bn254>;

//     // Native Types:
//     using Proof = plonk::proof;
//     using VK = std::shared_ptr<bonk::verification_key>;

//     static AggregationObject aggregate(Composer* composer,
//                                        const VK& vk,
//                                        const Proof& proof,
//                                        const AggregationObject recursion_output = AggregationObject())
//     {
//         std::shared_ptr<verification_key<bn254>> recursive_vk = verification_key<bn254>::from_witness(composer, vk);
//         const transcript::Manifest recursive_manifest = Composer::create_manifest(vk->num_public_inputs);

//         AggregationObject result = verify_proof<bn254, recursive_inner_verifier_settings<bn254>>(
//             composer, recursive_vk, recursive_manifest, proof, recursion_output);

//         return result;
//     }
// };

class Aggregator {
  public:
    static CT::AggregationObject aggregate(Composer* composer,
                                           const std::shared_ptr<CT::VK>& vk,
                                           const NT::Proof& proof,
                                           const size_t& num_public_inputs,
                                           const CT::AggregationObject recursion_output = CT::AggregationObject())
    {
        const Manifest recursive_manifest = Composer::create_manifest(num_public_inputs);

        CT::AggregationObject result = verify_proof<CT::bn254, recursive_inner_verifier_settings>(
            composer, vk, recursive_manifest, proof, recursion_output);

        return result;
    }
};
} // namespace aztec3::circuits::recursion