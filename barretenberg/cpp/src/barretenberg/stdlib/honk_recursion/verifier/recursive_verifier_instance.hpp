#pragma once
#include "barretenberg/commitment_schemes/verification_key.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/sumcheck/instance/verifier_instance.hpp"

namespace bb::stdlib::recursion::honk {

/**
 * @brief The stdlib counterpart of VerifierInstance, used in recursive folding verification.
 */
template <IsRecursiveFlavor Flavor> class RecursiveVerifierInstance_ {
  public:
    using FF = typename Flavor::FF;
    using NativeFF = typename Flavor::Curve::ScalarFieldNative;
    using Commitment = typename Flavor::Commitment;
    using VerificationKey = typename Flavor::VerificationKey;
    using NativeVerificationKey = typename Flavor::NativeVerificationKey;
    using WitnessCommitments = typename Flavor::WitnessCommitments;
    using CommitmentLabels = typename Flavor::CommitmentLabels;
    using RelationSeparator = typename Flavor::RelationSeparator;
    using Builder = typename Flavor::CircuitBuilder;
    using NativeFlavor = typename Flavor::NativeFlavor;
    using VerifierInstance = bb::VerifierInstance_<NativeFlavor>;

    Builder* builder;

    std::shared_ptr<VerificationKey> verification_key;
    RelationParameters<FF> relation_parameters;
    RelationSeparator alphas;
    bool is_accumulator = false;
    std::vector<FF> public_inputs;

    // The folding parameters (\vec{β}, e) which are set for accumulators (i.e. relaxed instances).
    std::vector<FF> gate_challenges;
    FF target_sum;

    WitnessCommitments witness_commitments;
    CommitmentLabels commitment_labels;

    RecursiveVerifierInstance_(Builder* builder)
        : builder(builder){};
    RecursiveVerifierInstance_(Builder* builder, std::shared_ptr<NativeVerificationKey> vk)
        : builder(builder)
        , verification_key(std::make_shared<VerificationKey>(builder, vk))
    {}

    RecursiveVerifierInstance_(Builder* builder, const std::shared_ptr<VerifierInstance>& instance)
        : verification_key(std::make_shared<VerificationKey>(instance->verification_key->circuit_size,
                                                             instance->verification_key->num_public_inputs))
        , is_accumulator(bool(instance->is_accumulator))
        , public_inputs(std::vector<FF>(instance->verification_key->num_public_inputs))
    {

        verification_key->pub_inputs_offset = instance->verification_key->pub_inputs_offset;
        verification_key->pcs_verification_key = instance->verification_key->pcs_verification_key;

        for (auto [public_input, native_public_input] : zip_view(public_inputs, instance->public_inputs)) {
            public_input = FF::from_witness(builder, native_public_input);
        }

        auto other_vks = instance->verification_key->get_all();
        size_t vk_idx = 0;
        for (auto& vk : verification_key->get_all()) {
            vk = Commitment::from_witness(builder, other_vks[vk_idx]);
            vk_idx++;
        }
        for (size_t alpha_idx = 0; alpha_idx < alphas.size(); alpha_idx++) {
            alphas[alpha_idx] = FF::from_witness(builder, instance->alphas[alpha_idx]);
        }

        auto other_comms = instance->witness_commitments.get_all();
        size_t comm_idx = 0;
        for (auto& comm : witness_commitments.get_all()) {
            comm = Commitment::from_witness(builder, other_comms[comm_idx]);
            comm_idx++;
        }
        target_sum = FF::from_witness(builder, instance->target_sum);

        size_t challenge_idx = 0;
        gate_challenges = std::vector<FF>(instance->gate_challenges.size());
        for (auto& challenge : gate_challenges) {
            challenge = FF::from_witness(builder, instance->gate_challenges[challenge_idx]);
            challenge_idx++;
        }
        relation_parameters.eta = FF::from_witness(builder, instance->relation_parameters.eta);
        relation_parameters.beta = FF::from_witness(builder, instance->relation_parameters.beta);
        relation_parameters.gamma = FF::from_witness(builder, instance->relation_parameters.gamma);
        relation_parameters.public_input_delta =
            FF::from_witness(builder, instance->relation_parameters.public_input_delta);
        relation_parameters.lookup_grand_product_delta =
            FF::from_witness(builder, instance->relation_parameters.lookup_grand_product_delta);
    }

    /**
     * @brief Return the underlying native VerifierInstance.
     *
     * @details In the context of client IVC, we will have several iterations of recursive folding verification. The
     * RecursiveVerifierInstance is tied to the builder in whose context it was created so in order to preserve the
     * accumulator values between several iterations we need to retrieve the native VerifierInstance values.
     */
    VerifierInstance get_value()
    {
        auto inst_verification_key = std::make_shared<NativeVerificationKey>(verification_key->circuit_size,
                                                                             verification_key->num_public_inputs);
        inst_verification_key->pcs_verification_key = verification_key->pcs_verification_key;
        for (auto [vk, inst_vk] : zip_view(verification_key->get_all(), inst_verification_key->get_all())) {
            inst_vk = vk.get_value();
        }

        VerifierInstance inst(inst_verification_key);
        inst.is_accumulator = is_accumulator;

        inst.public_inputs = std::vector<NativeFF>(verification_key->num_public_inputs);
        for (auto [public_input, inst_public_input] : zip_view(public_inputs, inst.public_inputs)) {
            inst_public_input = public_input.get_value();
        }

        for (auto [alpha, inst_alpha] : zip_view(alphas, inst.alphas)) {
            inst_alpha = alpha.get_value();
        }

        for (auto [comm, inst_comm] : zip_view(witness_commitments.get_all(), inst.witness_commitments.get_all())) {
            inst_comm = comm.get_value();
        }
        inst.target_sum = target_sum.get_value();

        inst.gate_challenges = std::vector<NativeFF>(gate_challenges.size());
        for (auto [challenge, inst_challenge] : zip_view(gate_challenges, inst.gate_challenges)) {
            inst_challenge = challenge.get_value();
        }

        inst.relation_parameters.eta = relation_parameters.eta.get_value();
        inst.relation_parameters.beta = relation_parameters.beta.get_value();
        inst.relation_parameters.gamma = relation_parameters.gamma.get_value();
        inst.relation_parameters.public_input_delta = relation_parameters.public_input_delta.get_value();
        inst.relation_parameters.lookup_grand_product_delta =
            relation_parameters.lookup_grand_product_delta.get_value();
        return inst;
    }
};
} // namespace bb::stdlib::recursion::honk