#include "protogalaxy_prover.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
namespace proof_system::honk {
template <class Flavor>
ProtoGalaxyProver_<Flavor>::ProtoGalaxyProver_(std::vector<std::shared_ptr<Instance>> insts)
    : instances(insts)
{}

/**
 * @brief Prior to folding we need to add all the public inputs to the transcript, labelled by their corresponding
 * instance index, compute all the instance's polynomials and record the relation parameters involved in computing these
 * polynomials in the transcript.
 *
 */
template <class Flavor> void ProtoGalaxyProver_<Flavor>::prepare_for_folding()
{
    for (const auto& instance : instances) {
        instance->initialise_prover_polynomials();

        const auto instance_index = std::to_string(instance->index);
        const auto circuit_size = static_cast<uint32_t>(instance->proving_key->circuit_size);
        const auto num_public_inputs = static_cast<uint32_t>(instance->proving_key->num_public_inputs);

        transcript.send_to_verifier(instance_index + "_circuit_size", circuit_size);
        transcript.send_to_verifier(instance_index + "_public_input_size", num_public_inputs);
        transcript.send_to_verifier(instance_index + "_pub_inputs_offset",
                                    static_cast<uint32_t>(instance->pub_inputs_offset));

        for (size_t i = 0; i < instance->proving_key->num_public_inputs; ++i) {
            auto public_input_i = instance->public_inputs[i];
            transcript.send_to_verifier(instance_index + "_public_input_" + std::to_string(i), public_input_i);
        }

        auto [eta, beta, gamma] =
            transcript.get_challenges(instance_index + "_eta", instance_index + "_beta", instance_index + "_gamma");
        instance->compute_sorted_accumulator_polynomials(eta);
        instance->compute_grand_product_polynomials(beta, gamma);
    }
}

// TODO(#689): implement this function
template <class Flavor> ProverFoldingResult<Flavor> ProtoGalaxyProver_<Flavor>::fold_instances()
{
    prepare_for_folding();
    ProverFoldingResult<Flavor> res;
    res.folding_data = transcript.proof_data;
    return res;
}

template class ProtoGalaxyProver_<honk::flavor::Ultra>;
template class ProtoGalaxyProver_<honk::flavor::UltraGrumpkin>;
template class ProtoGalaxyProver_<honk::flavor::GoblinUltra>;

} // namespace proof_system::honk