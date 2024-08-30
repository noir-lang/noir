#pragma once
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/protogalaxy/folding_result.hpp"

namespace bb {

template <class ProverInstances_> class ProtoGalaxyProver_ {
  public:
    using ProverInstance = typename ProverInstances_::Instance;
    using Flavor = typename ProverInstances_::Flavor;
    using FF = typename ProverInstances_::Flavor::FF;
    static constexpr size_t NUM_INSTANCES = ProverInstances_::NUM;
    using CombinerQuotient = Univariate<FF, ProverInstances_::BATCHED_EXTENDED_LENGTH, NUM_INSTANCES>;
    using TupleOfTuplesOfUnivariates = typename Flavor::template ProtogalaxyTupleOfTuplesOfUnivariates<NUM_INSTANCES>;
    using OptimisedTupleOfTuplesOfUnivariates =
        typename Flavor::template OptimisedProtogalaxyTupleOfTuplesOfUnivariates<NUM_INSTANCES>;
    using UnivariateRelationParameters =
        bb::RelationParameters<Univariate<FF, ProverInstances_::EXTENDED_LENGTH, 0, /*skip_count=*/NUM_INSTANCES - 1>>;
    using UnivariateRelationSeparator =
        std::array<Univariate<FF, ProverInstances_::BATCHED_EXTENDED_LENGTH>, Flavor::NUM_SUBRELATIONS - 1>;

    struct State {
        std::shared_ptr<ProverInstance> accumulator;
        Polynomial<FF> perturbator;
        std::vector<FF> deltas;
        CombinerQuotient combiner_quotient;
        FF perturbator_evaluation;
        UnivariateRelationParameters relation_parameters;
        UnivariateRelationSeparator alphas;
    };
    using Transcript = typename Flavor::Transcript;
    using Instance = typename ProverInstances_::Instance;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using ProverInstances = ProverInstances_;

    static constexpr size_t NUM_SUBRELATIONS = ProverInstances_::NUM_SUBRELATIONS;

    ProverInstances_ instances;
    std::shared_ptr<Transcript> transcript = std::make_shared<Transcript>();
    std::shared_ptr<CommitmentKey> commitment_key;
    State state;

    ProtoGalaxyProver_() = default;
    ProtoGalaxyProver_(const std::vector<std::shared_ptr<Instance>>& insts)
        : instances(ProverInstances_(insts))
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/878)
        , commitment_key(instances[1]->proving_key.commitment_key){};

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
     * @brief Create inputs to folding protocol (an Oink interaction).
     * @details Finalise the prover instances that will be folded: complete computation of all the witness polynomials
     * and compute commitments. Send commitments to the verifier and retrieve challenges.
     */
    void run_oink_prover_on_each_instance();

    /**
     * @brief Steps 2 - 5 of the paper.
     * @details Compute perturbator (F polynomial in paper). Send all but the constant coefficient to verifier.
     *
     * @param accumulator
     * @return std::tuple<std::vector<FF>, Polynomial<FF>> deltas, perturbator
     */
    std::tuple<std::vector<FF>, Polynomial<FF>> perturbator_round(const std::shared_ptr<const Instance>& accumulator);

    /**
     * @brief Steps 6 - 11 of the paper.
     * @details Compute combiner (G polynomial in the paper) and then its quotient (K polynomial), whose coefficient
     * will be sent to the verifier.
     */
    /*gate_challenges, alphas, optimised_relation_parameters, perturbator_evaluation, combiner_quotient */
    std::tuple<std::vector<FF>, UnivariateRelationSeparator, UnivariateRelationParameters, FF, CombinerQuotient>
    combiner_quotient_round(const std::vector<FF>& gate_challenges,
                            const std::vector<FF>& deltas,
                            const ProverInstances_& instances);

    /**
     * @brief Steps 12 - 13 of the paper plus the prover folding work.
     * @details Compute \f$ e^* \f$ plus, then update the prover accumulator by taking a Lagrange-linear combination of
     * the current accumulator and the instances to be folded. In our mental model, we are doing a scalar multipliation
     * of matrices whose columns are polynomials, as well as taking similar linear combinations of the relation
     * parameters.
     */
    FoldingResult<Flavor> update_target_sum_and_fold(const ProverInstances_& instances,
                                                     const CombinerQuotient& combiner_quotient,
                                                     const UnivariateRelationSeparator& alphas,
                                                     const UnivariateRelationParameters& univariate_relation_parameters,
                                                     const FF& perturbator_evaluation);
};
} // namespace bb