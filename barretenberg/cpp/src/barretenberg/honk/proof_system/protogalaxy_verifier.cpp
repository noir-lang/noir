#include "protogalaxy_verifier.hpp"
#include "barretenberg/honk/utils/grand_product_delta.hpp"
namespace proof_system::honk {
template <class Flavor>
ProtoGalaxyVerifier_<Flavor>::ProtoGalaxyVerifier_(std::vector<std::shared_ptr<VerificationKey>> vks)
{
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/391): simplify code with C++23 features
    uint32_t idx = 0;
    for (const auto& vk : vks) {
        VerifierInstance inst;
        inst.verification_key = std::move(vk);
        inst.index = idx;
        verifier_instances.emplace_back(inst);
        idx++;
    }
}

template <class Flavor>
VerifierFoldingResult<Flavor> ProtoGalaxyVerifier_<Flavor>::fold_public_parameters(std::vector<uint8_t> fold_data)
{
    transcript = VerifierTranscript<FF>{ fold_data };
    for (auto& inst : verifier_instances) {
        auto idx = std::to_string(inst.index);
        inst.circuit_size = transcript.template receive_from_prover<uint32_t>(idx + "_circuit_size");
        inst.public_input_size = transcript.template receive_from_prover<uint32_t>(idx + "_public_input_size");
        inst.pub_inputs_offset = transcript.template receive_from_prover<uint32_t>(idx + "_pub_inputs_offset");

        for (size_t i = 0; i < inst.public_input_size; ++i) {
            auto public_input_i =
                transcript.template receive_from_prover<FF>(idx + "_public_input_" + std::to_string(i));
            inst.public_inputs.emplace_back(public_input_i);
        }
        auto [eta, beta, gamma] = transcript.get_challenges(idx + "_eta", idx + "_beta", idx + "_gamma");
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

template class ProtoGalaxyVerifier_<honk::flavor::Ultra>;
template class ProtoGalaxyVerifier_<honk::flavor::UltraGrumpkin>;
template class ProtoGalaxyVerifier_<honk::flavor::GoblinUltra>;
} // namespace proof_system::honk