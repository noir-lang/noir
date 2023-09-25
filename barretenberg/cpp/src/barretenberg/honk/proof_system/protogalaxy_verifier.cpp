#include "protogalaxy_verifier.hpp"
#include "barretenberg/honk/utils/grand_product_delta.hpp"
namespace proof_system::honk {
template <class VerifierInstances>
VerifierFoldingResult<typename VerifierInstances::Flavor> ProtoGalaxyVerifier_<
    VerifierInstances>::fold_public_parameters(std::vector<uint8_t> fold_data)
{
    transcript = VerifierTranscript<FF>{ fold_data };
    auto index = 0;
    for (auto it = verifier_instances.begin(); it != verifier_instances.end(); it++, index++) {
        auto inst = *it;
        auto domain_separator = std::to_string(index);
        inst.circuit_size = transcript.template receive_from_prover<uint32_t>(domain_separator + "_circuit_size");
        inst.public_input_size =
            transcript.template receive_from_prover<uint32_t>(domain_separator + "_public_input_size");
        inst.pub_inputs_offset =
            transcript.template receive_from_prover<uint32_t>(domain_separator + "_pub_inputs_offset");

        for (size_t i = 0; i < inst.public_input_size; ++i) {
            auto public_input_i =
                transcript.template receive_from_prover<FF>(domain_separator + "_public_input_" + std::to_string(i));
            inst.public_inputs.emplace_back(public_input_i);
        }
        auto [eta, beta, gamma] = transcript.get_challenges(
            domain_separator + "_eta", domain_separator + "_beta", domain_separator + "_gamma");
        const FF public_input_delta = compute_public_input_delta<Flavor>(
            inst.public_inputs, beta, gamma, inst.circuit_size, inst.pub_inputs_offset);
        const FF lookup_grand_product_delta = compute_lookup_grand_product_delta<FF>(beta, gamma, inst.circuit_size);
        inst.relation_parameters =
            RelationParameters<FF>{ eta, beta, gamma, public_input_delta, lookup_grand_product_delta };
    }

    // TODO(#690): implement the  Protogalaxy verifier logic
    VerifierFoldingResult<Flavor> res;
    return res;
}

template class ProtoGalaxyVerifier_<VerifierInstances_<honk::flavor::Ultra, 2>>;
template class ProtoGalaxyVerifier_<VerifierInstances_<honk::flavor::UltraGrumpkin, 2>>;
template class ProtoGalaxyVerifier_<VerifierInstances_<honk::flavor::GoblinUltra, 2>>;
} // namespace proof_system::honk