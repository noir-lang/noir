#pragma once
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/protogalaxy/folding_result.hpp"

namespace bb {
template <class ProverInstances_> struct ProtogalaxyProofConstructionState {
    using FF = typename ProverInstances_::FF;
    using ProverInstance = typename ProverInstances_::Instance;
    using Flavor = typename ProverInstances_::Flavor;
    using TupleOfTuplesOfUnivariates =
        typename Flavor::template ProtogalaxyTupleOfTuplesOfUnivariates<ProverInstances_::NUM>;
    using OptimisedTupleOfTuplesOfUnivariates =
        typename Flavor::template OptimisedProtogalaxyTupleOfTuplesOfUnivariates<ProverInstances_::NUM>;

    std::shared_ptr<ProverInstance> accumulator;
    LegacyPolynomial<FF> perturbator;
    std::vector<FF> deltas;
    Univariate<FF, ProverInstances_::BATCHED_EXTENDED_LENGTH, ProverInstances_::NUM> combiner_quotient;
    FF compressed_perturbator;
    OptimisedTupleOfTuplesOfUnivariates optimised_univariate_accumulators;
    TupleOfTuplesOfUnivariates univariate_accumulators;
    FoldingResult<typename ProverInstances_::Flavor> result;
};

template <class ProverInstances_> class ProtoGalaxyProver_ {
  public:
    using ProverInstances = ProverInstances_;
    using Flavor = typename ProverInstances::Flavor;
    using Transcript = typename Flavor::Transcript;
    using FF = typename Flavor::FF;
    using Instance = typename ProverInstances::Instance;
    using CommitmentKey = typename Flavor::CommitmentKey;

    static constexpr size_t NUM_SUBRELATIONS = ProverInstances::NUM_SUBRELATIONS;

    ProverInstances instances;
    std::shared_ptr<Transcript> transcript = std::make_shared<Transcript>();
    std::shared_ptr<CommitmentKey> commitment_key;
    ProtogalaxyProofConstructionState<ProverInstances> state;

    ProtoGalaxyProver_() = default;
    ProtoGalaxyProver_(const std::vector<std::shared_ptr<Instance>>& insts)
        : instances(ProverInstances(insts))
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/878)
        , commitment_key(instances[1]->proving_key.commitment_key){};
    ~ProtoGalaxyProver_() = default;

    /**
     * @brief Prior to folding, we need to finalize the given instances and add all their public data ϕ to the
     * transcript, labelled by their corresponding instance index for domain separation.
     */
    void prepare_for_folding();

    /**
     * @brief For each instance produced by a circuit, prior to folding, we need to complete the computation of its
     * prover polynomials, commit to witnesses and generate the relation parameters as well as send the public data ϕ of
     * an instance to the verifier.
     *
     * @param domain_separator  separates the same type of data coming from difference instances by instance
     * index
     */
    void finalise_and_send_instance(std::shared_ptr<Instance>, const std::string& domain_separator);

    /**
     * @brief Execute the folding prover.
     *
     * @todo TODO(https://github.com/AztecProtocol/barretenberg/issues/753): fold goblin polynomials
     * @return FoldingResult is a pair consisting of an accumulator and a folding proof, which is a proof that the
     * accumulator was computed correctly.
     */
    BB_PROFILE FoldingResult<Flavor> prove();

    // Returns the accumulator, which is the first element in ProverInstances. The accumulator is assumed to have the
    // FoldingParameters set and be the result of a previous round of folding.
    std::shared_ptr<Instance> get_accumulator() { return instances[0]; }

    /**
     * @brief Compute the next accumulator (ϕ*, ω*, \vec{\beta*}, e*), send the public data ϕ*  and the folding
     * parameters
     * (\vec{\beta*}, e*) to the verifier and return the complete accumulator
     *
     * @details At this stage, we assume that the instances have the same size and the same number of public
     * parameter.s
     * @param instances
     * @param combiner_quotient polynomial K in the paper
     * @param challenge
     * @param compressed_perturbator
     *
     * TODO(https://github.com/AztecProtocol/barretenberg/issues/796): optimise the construction of the new
     * accumulator
     */
    std::shared_ptr<Instance> compute_next_accumulator(
        ProverInstances& instances,
        Univariate<FF, ProverInstances::BATCHED_EXTENDED_LENGTH, ProverInstances::NUM>& combiner_quotient,
        FF& challenge,
        const FF& compressed_perturbator);

    /**
     * @brief Finalise the prover instances that will be folded: complete computation of all the witness polynomials
     * and compute commitments. Send commitments to the verifier and retrieve challenges.
     *
     */
    void preparation_round();

    /**
     * @brief Compute perturbator (F polynomial in paper). Send all but the constant coefficient to verifier.
     *
     */
    void perturbator_round();

    /**
     * @brief Compute combiner (G polynomial in the paper) and then its quotient (K polynomial), whose coefficient
     * will be sent to the verifier.
     *
     */
    void combiner_quotient_round();

    /**
     * @brief Compute the next prover accumulator (ω* in the paper), encapsulated in a ProverInstance with folding
     * parameters set.
     *
     */
    void accumulator_update_round();
};
} // namespace bb