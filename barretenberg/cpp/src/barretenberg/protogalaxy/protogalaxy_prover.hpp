#pragma once
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/protogalaxy/folding_result.hpp"

namespace bb {

template <class ProverInstances_> class ProtoGalaxyProver_ {
  public:
    struct State {
        using FF = typename ProverInstances_::FF;
        using ProverInstance = typename ProverInstances_::Instance;
        using Flavor = typename ProverInstances_::Flavor;
        static constexpr size_t NUM_INSTANCES = ProverInstances_::NUM;
        using CombinerQuotient = Univariate<FF, ProverInstances_::BATCHED_EXTENDED_LENGTH, NUM_INSTANCES>;
        using TupleOfTuplesOfUnivariates =
            typename Flavor::template ProtogalaxyTupleOfTuplesOfUnivariates<NUM_INSTANCES>;
        using OptimisedTupleOfTuplesOfUnivariates =
            typename Flavor::template OptimisedProtogalaxyTupleOfTuplesOfUnivariates<NUM_INSTANCES>;
        using RelationParameters = bb::RelationParameters<Univariate<FF, ProverInstances_::EXTENDED_LENGTH>>;
        using OptimisedRelationParameters = bb::RelationParameters<
            Univariate<FF, ProverInstances_::EXTENDED_LENGTH, 0, /*skip_count=*/NUM_INSTANCES - 1>>;
        using CombinedRelationSeparator =
            std::array<Univariate<FF, ProverInstances_::BATCHED_EXTENDED_LENGTH>, Flavor::NUM_SUBRELATIONS - 1>;

        std::shared_ptr<ProverInstance> accumulator;
        LegacyPolynomial<FF> perturbator;
        std::vector<FF> gate_challenges;
        std::vector<FF> deltas;
        CombinerQuotient combiner_quotient;
        FF compressed_perturbator;
        RelationParameters relation_parameters;
        CombinedRelationSeparator alphas; // a univariate interpolation of challenges for each subrelation
        OptimisedRelationParameters optimised_relation_parameters;
        OptimisedTupleOfTuplesOfUnivariates optimised_univariate_accumulators;
        TupleOfTuplesOfUnivariates univariate_accumulators;
        FoldingResult<typename ProverInstances_::Flavor> result;
    };

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
    State state;

    ProtoGalaxyProver_() = default;
    ProtoGalaxyProver_(const std::vector<std::shared_ptr<Instance>>& insts)
        : instances(ProverInstances(insts))
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/878)
        , commitment_key(instances[1]->proving_key.commitment_key){};

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
    std::shared_ptr<Instance> compute_next_accumulator(ProverInstances&,
                                                       State::CombinerQuotient&,
                                                       State::OptimisedRelationParameters&,
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