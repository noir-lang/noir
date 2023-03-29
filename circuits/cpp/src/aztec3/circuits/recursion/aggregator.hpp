#pragma once
#include <barretenberg/stdlib/types/types.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <barretenberg/stdlib/recursion/verifier/verifier.hpp>
#include <barretenberg/stdlib/recursion/verifier/program_settings.hpp>
#include <barretenberg/transcript/manifest.hpp>

namespace aztec3::circuits::recursion {

// These are all Circuit Types!
// using plonk::stdlib::recursion::aggregation_state;
using plonk::stdlib::types::recursive_inner_verifier_settings;
// using plonk::stdlib::recursion::verification_key;
using plonk::stdlib::recursion::verify_proof;
// using aztec3::utils::types::bn254;
using plonk::stdlib::types::Composer;
using CT = aztec3::utils::types::CircuitTypes<Composer>;
using NT = aztec3::utils::types::NativeTypes;
using transcript::Manifest;

class Aggregator {
  public:
    static CT::AggregationObject aggregate(
        Composer* composer,
        const std::shared_ptr<CT::VK>& vk,
        const NT::Proof& proof,
        const size_t& num_public_inputs,
        const CT::AggregationObject previous_aggregation_output = CT::AggregationObject())
    {
        const Manifest recursive_manifest = Composer::create_manifest(num_public_inputs);

        CT::AggregationObject result = verify_proof<CT::bn254, recursive_inner_verifier_settings>(
            composer, vk, recursive_manifest, proof, previous_aggregation_output);

        return result;
    }
};
} // namespace aztec3::circuits::recursion