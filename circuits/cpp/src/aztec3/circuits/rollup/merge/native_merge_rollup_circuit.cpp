#include "aztec3/constants.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/crypto/sha256/sha256.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/merkle_tree/membership.hpp"
#include "barretenberg/stdlib/merkle_tree/memory_tree.hpp"
#include "barretenberg/stdlib/merkle_tree/merkle_tree.hpp"
#include "init.hpp"

#include <algorithm>
#include <array>
#include <cstdint>
#include <tuple>
#include <vector>

namespace aztec3::circuits::rollup::native_merge_rollup {

/**
 * @brief Create an aggregation object for the proofs that are provided
 *          - We add points P0 for each of our proofs
 *          - We add points P1 for each of our proofs
 *          - We concat our public inputs
 *
 * @param mergeRollupInputs
 * @return AggregationObject
 */
AggregationObject aggregate_proofs(MergeRollupInputs mergeRollupInputs)
{

    // TODO: NOTE: for now we simply return the aggregation object from the first proof
    return mergeRollupInputs.previous_rollup_data[0].merge_rollup_public_inputs.end_aggregation_object;
}

/** TODO: implement
 * @brief Get the prover contribution hash object
 *
 * @return NT::fr
 */
NT::fr get_prover_contribution_hash()
{
    return NT::fr(0);
}

MergeRollupPublicInputs merge_rollup_circuit(MergeRollupInputs mergeRollupInputs)
{
    // Verify the previous rollup proofs

    AggregationObject aggregation_object = aggregate_proofs(mergeRollupInputs);

    MergeRollupPublicInputs public_inputs = {};
    return public_inputs;
}

} // namespace aztec3::circuits::rollup::native_merge_rollup