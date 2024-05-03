#pragma once
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/protogalaxy/folding_result.hpp"
#include "barretenberg/stdlib/honk_recursion/transcript/transcript.hpp"
#include "barretenberg/stdlib/honk_recursion/verifier/recursive_instances.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_recursive_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_recursive_flavor.hpp"

namespace bb::stdlib::recursion::honk {
template <class VerifierInstances> class ProtoGalaxyRecursiveVerifier_ {
  public:
    using Flavor = typename VerifierInstances::Flavor;
    using NativeFlavor = typename Flavor::NativeFlavor;
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using GroupElement = typename Flavor::GroupElement;
    using Instance = typename VerifierInstances::Instance;
    using NativeInstance = bb::VerifierInstance_<NativeFlavor>;
    using VerificationKey = typename Flavor::VerificationKey;
    using NativeVerificationKey = typename Flavor::NativeVerificationKey;
    using WitnessCommitments = typename Flavor::WitnessCommitments;
    using CommitmentLabels = typename Flavor::CommitmentLabels;
    using Builder = typename Flavor::CircuitBuilder;
    using RelationSeparator = typename Flavor::RelationSeparator;
    using PairingPoints = std::array<GroupElement, 2>;
    static constexpr size_t NUM = VerifierInstances::NUM;
    using Transcript = bb::BaseTranscript<bb::stdlib::recursion::honk::StdlibTranscriptParams<Builder>>;

    static constexpr size_t NUM_SUBRELATIONS = Flavor::NUM_SUBRELATIONS;

    CommitmentLabels commitment_labels;

    Builder* builder;
    std::shared_ptr<Transcript> transcript;
    VerifierInstances instances;

    ProtoGalaxyRecursiveVerifier_(Builder* builder,
                                  std::shared_ptr<NativeInstance>& accumulator,
                                  const std::vector<std::shared_ptr<NativeVerificationKey>>& native_inst_vks)
        : builder(builder)
        , instances(VerifierInstances(builder, accumulator, native_inst_vks)){};

    /**
     * @brief Given a new round challenge δ for each iteration of the full ProtoGalaxy protocol, compute the vector
     * [δ, δ^2,..., δ^t] where t = logn and n is the size of the instance.
     */
    static std::vector<FF> compute_round_challenge_pows(size_t log_instance_size, FF round_challenge)
    {
        std::vector<FF> pows(log_instance_size);
        pows[0] = round_challenge;
        for (size_t i = 1; i < log_instance_size; i++) {
            pows[i] = pows[i - 1].sqr();
        }
        return pows;
    }

    static std::vector<FF> update_gate_challenges(const FF perturbator_challenge,
                                                  const std::vector<FF>& gate_challenges,
                                                  const std::vector<FF>& round_challenges)
    {
        auto log_instance_size = gate_challenges.size();
        std::vector<FF> next_gate_challenges(log_instance_size);

        for (size_t idx = 0; idx < log_instance_size; idx++) {
            next_gate_challenges[idx] = gate_challenges[idx] + perturbator_challenge * round_challenges[idx];
        }
        return next_gate_challenges;
    }

    std::shared_ptr<Instance> get_accumulator() { return instances[0]; }

    /**
     * @brief Instatiate the instances and the transcript.
     *
     * @param fold_data The data transmitted via the transcript by the prover.
     */
    void prepare_for_folding();

    /**
     * @brief Instantiate the accumulator (i.e. the relaxed instance) from the transcript.
     *
     */
    void receive_accumulator(const std::shared_ptr<Instance>&, const std::string&);

    /**
     * @brief Process the public data ϕ for the Instances to be folded.
     *
     */
    void receive_and_finalise_instance(const std::shared_ptr<Instance>&, const std::string&);

    /**
     * @brief Run the folding protocol on the verifier side to establish whether the public data ϕ of the new
     * accumulator, received from the prover is the same as that produced by the verifier.
     *
     * @details In the recursive setting this function doesn't return anything because the equality checks performed by
     * the recursive verifier, ensuring the folded ϕ*, e* and β* on the verifier side correspond to what has been sent
     * by the prover, are expressed as constraints.

     */
    std::shared_ptr<Instance> verify_folding_proof(const HonkProof&);

    /**
     * @brief Evaluates the perturbator at a  given scalar, in a sequential manner for the recursive setting.
     *
     * @details This method is equivalent to the one in the Polynomial class for evaluating a polynomial, represented by
     * coefficients in monomial basis, at a given point. The Polynomial class is used in the native verifier for
     * constructing and computing the perturbator. We implement this separate functionality here in the recursive
     * folding verifier to avoid instantiating the entire Polynomial class on stdlib::bn254. Furthermore, the evaluation
     * needs to be done sequentially as we don't support a parallel_for in circuits.
     *
     */
    static FF evaluate_perturbator(std::vector<FF> coeffs, FF point)
    {
        FF point_acc = FF(1);
        FF result = FF(0);
        for (size_t i = 0; i < coeffs.size(); i++) {
            result += coeffs[i] * point_acc;
            point_acc *= point;
        }
        return result;
    };

    /**
     * @brief Hack method to fold the witness commitments and verification key without the batch_mul in the case where
     * the recursive folding verifier is instantiated as a vanilla ultra circuit.
     *
     * @details In the folding recursive verifier we might hit the scenerio where we do a batch_mul(commitments,
     * lagranges) where the commitments are equal. That is because when we add gates to ensure no zero commitments,
     * these will be the same for all circuits, hitting an edge case in batch_mul that creates a failing constraint.
     * Specifically, at some point in the algorithm we compute the difference between the points which, if they are
     * equal, would be zero, case that is not supported. See https://github.com/AztecProtocol/barretenberg/issues/971.
     */
    void fold_commitments(std::vector<FF> lagranges,
                          VerifierInstances& instances,
                          std::shared_ptr<Instance>& accumulator)
        requires IsUltraBuilder<Builder>
    {
        using ElementNative = typename Flavor::Curve::ElementNative;
        using AffineElementNative = typename Flavor::Curve::AffineElementNative;

        auto offset_generator = Commitment::from_witness(builder, AffineElementNative(ElementNative::random_element()));

        size_t vk_idx = 0;
        for (auto& expected_vk : accumulator->verification_key->get_all()) {
            expected_vk = offset_generator;
            size_t inst = 0;
            for (auto& instance : instances) {
                expected_vk += instance->verification_key->get_all()[vk_idx] * lagranges[inst];
                inst++;
            }
            expected_vk -= offset_generator;
            vk_idx++;
        }

        size_t comm_idx = 0;
        for (auto& comm : accumulator->witness_commitments.get_all()) {
            comm = offset_generator;
            size_t inst = 0;
            for (auto& instance : instances) {
                comm += instance->witness_commitments.get_all()[comm_idx] * lagranges[inst];
                inst++;
            }
            comm -= offset_generator;
            comm_idx++;
        }
    }

    /**
     * @brief Folds the witness commitments and verification key (part of ϕ) and stores the values in the accumulator.
     *
     *
     */

    void fold_commitments(std::vector<FF> lagranges,
                          VerifierInstances& instances,
                          std::shared_ptr<Instance>& accumulator)
        requires(!IsUltraBuilder<Builder>)
    {
        size_t vk_idx = 0;
        for (auto& expected_vk : accumulator->verification_key->get_all()) {
            std::vector<Commitment> commitments;
            for (auto& instance : instances) {
                commitments.emplace_back(instance->verification_key->get_all()[vk_idx]);
            }
            expected_vk = Commitment::batch_mul(commitments, lagranges);
            vk_idx++;
        }

        size_t comm_idx = 0;
        for (auto& comm : accumulator->witness_commitments.get_all()) {
            std::vector<Commitment> commitments;
            for (auto& instance : instances) {
                commitments.emplace_back(instance->witness_commitments.get_all()[comm_idx]);
            }
            comm = Commitment::batch_mul(commitments, lagranges);
            comm_idx++;
        }
    }
};

} // namespace bb::stdlib::recursion::honk