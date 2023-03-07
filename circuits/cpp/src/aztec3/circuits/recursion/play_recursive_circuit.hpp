// #include <stdlib/types/types.hpp>
// #include <stdlib/recursion/verifier/verifier.hpp>
// #include <stdlib/recursion/verifier/program_settings.hpp>
// #include <plonk/transcript/manifest.hpp>

namespace aztec3::circuits::recursion {

using plonk::stdlib::types::Composer;
using CT = plonk::stdlib::types::CircuitTypes<Composer>;
using NT = plonk::stdlib::types::NativeTypes;

CT::AggregationObject play_recursive_circuit(Composer& composer,
                                             std::shared_ptr<NT::VK> const& app_vk,
                                             NT::Proof const& app_proof)
{
    std::shared_ptr<CT::VK> app_vk_ct = CT::VK::from_witness(&composer, app_vk);

    CT::AggregationObject recursion_output =
        Aggregator::aggregate(&composer, app_vk_ct, app_proof, app_vk->num_public_inputs);

    recursion_output.add_proof_outputs_as_public_inputs();

    return recursion_output;
};

void dummy_circuit(Composer& composer, barretenberg::fr const& a_in, barretenberg::fr const& b_in)
{

    field_ct a = witness_ct(&composer, a_in);
    field_ct b = witness_ct(&composer, b_in);
    field_ct c = a * b;

    c.set_public();
};

CT::AggregationObject play_recursive_circuit_2(Composer& composer,
                                               std::shared_ptr<NT::VK> const& app_vk,
                                               NT::Proof const& app_proof,
                                               std::shared_ptr<NT::VK> const& prev_recursive_vk,
                                               NT::Proof const& prev_recursive_proof)
{
    std::shared_ptr<CT::VK> app_vk_ct = CT::VK::from_witness(&composer, app_vk);
    std::shared_ptr<CT::VK> prev_recursive_vk_ct = CT::VK::from_witness(&composer, prev_recursive_vk);

    info("composer failed 1? ", composer.failed());

    CT::AggregationObject aggregation_object = Aggregator::aggregate(
        &composer, prev_recursive_vk_ct, prev_recursive_proof, prev_recursive_vk->num_public_inputs);

    info("composer failed 2? ", composer.failed());

    info("\npublic inputs before: ", composer.public_inputs.size());
    aggregation_object =
        Aggregator::aggregate(&composer, app_vk_ct, app_proof, app_vk->num_public_inputs, aggregation_object);
    info("\npublic inputs after: ", composer.public_inputs.size());

    info("composer failed 3? ", composer.failed());

    aggregation_object.add_proof_outputs_as_public_inputs();

    info("composer failed 4? ", composer.failed());

    info("\npublic inputs after after: ", composer.public_inputs.size());

    info("composer failed 5? ", composer.failed());

    return aggregation_object;
};

} // namespace aztec3::circuits::recursion