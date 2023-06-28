#pragma once
#include "init.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::recursion {

class Aggregator {
  public:
    static CT::AggregationObject aggregate(
        Builder* builder,
        const std::shared_ptr<CT::VK>& vk,
        const NT::Proof& proof,
        const CT::AggregationObject& previous_aggregation_output = CT::AggregationObject())
    {
        CT::AggregationObject result =
            verify_proof<plonk::flavor::Ultra>(builder, vk, proof, previous_aggregation_output);

        return result;
    }
};
}  // namespace aztec3::circuits::recursion