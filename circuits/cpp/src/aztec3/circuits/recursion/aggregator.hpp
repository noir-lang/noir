#pragma once
#include "init.hpp"

namespace aztec3::circuits::recursion {

class Aggregator {
  public:
    static CT::AggregationObject aggregate(
        Composer* composer,
        const std::shared_ptr<CT::VK>& vk,
        const NT::Proof& proof,
        const size_t& num_public_inputs,
        const CT::AggregationObject& previous_aggregation_output = CT::AggregationObject())
    {
        const Manifest recursive_manifest = Composer::create_manifest(num_public_inputs);

        CT::AggregationObject result = verify_proof<CT::bn254, CT::recursive_inner_verifier_settings>(
            composer, vk, recursive_manifest, proof, previous_aggregation_output);

        return result;
    }
};
}  // namespace aztec3::circuits::recursion