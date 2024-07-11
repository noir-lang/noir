#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/polynomials/pow.hpp"
#include "barretenberg/protogalaxy/decider_verifier.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/protogalaxy/protogalaxy_verifier.hpp"
#include "barretenberg/stdlib_circuit_builders/mock_circuits.hpp"
#include "barretenberg/ultra_honk/decider_prover.hpp"

#include <gtest/gtest.h>

using namespace bb;

namespace {

auto& engine = numeric::get_debug_randomness();

template <typename Flavor> class ProtoGalaxyTests : public testing::Test {
  public:
    using VerificationKey = typename Flavor::VerificationKey;
    using ProverInstance = ProverInstance_<Flavor>;
    using ProverInstances = ProverInstances_<Flavor, 2>;
    using VerifierInstance = VerifierInstance_<Flavor>;
    using VerifierInstances = VerifierInstances_<Flavor, 2>;
    using ProtoGalaxyProver = ProtoGalaxyProver_<ProverInstances>;
    using FF = typename Flavor::FF;
    using Affine = typename Flavor::Commitment;
    using Projective = typename Flavor::GroupElement;
    using Builder = typename Flavor::CircuitBuilder;
    using Polynomial = typename Flavor::Polynomial;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using RelationParameters = bb::RelationParameters<FF>;
    using WitnessCommitments = typename Flavor::WitnessCommitments;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using PowPolynomial = bb::PowPolynomial<FF>;
    using DeciderProver = DeciderProver_<Flavor>;
    using DeciderVerifier = DeciderVerifier_<Flavor>;
    using FoldingProver = ProtoGalaxyProver_<ProverInstances>;
    using FoldingVerifier = ProtoGalaxyVerifier_<VerifierInstances>;

    using TupleOfInstances =
        std::tuple<std::vector<std::shared_ptr<ProverInstance>>, std::vector<std::shared_ptr<VerifierInstance>>>;

    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }

    static void construct_circuit(Builder& builder)
    {
        MockCircuits::add_arithmetic_gates(builder);
        if constexpr (IsGoblinFlavor<Flavor>) {
            GoblinMockCircuits::add_some_ecc_op_gates(builder);
        }
    }

    // Construct prover and verifier instance for a provided circuit and add to tuple
    static void construct_prover_and_verifier_instance(TupleOfInstances& instances,
                                                       Builder& builder,
                                                       TraceStructure structure = TraceStructure::NONE)
    {

        auto prover_instance = std::make_shared<ProverInstance>(builder, structure);
        auto verification_key = std::make_shared<VerificationKey>(prover_instance->proving_key);
        auto verifier_instance = std::make_shared<VerifierInstance>(verification_key);
        get<0>(instances).emplace_back(prover_instance);
        get<1>(instances).emplace_back(verifier_instance);
    }

    // constructs num_insts number of prover and verifier instances
    static TupleOfInstances construct_instances(size_t num_insts, TraceStructure structure = TraceStructure::NONE)
    {
        TupleOfInstances instances;
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/938): Parallelize this loop
        for (size_t idx = 0; idx < num_insts; idx++) {
            auto builder = typename Flavor::CircuitBuilder();
            construct_circuit(builder);

            construct_prover_and_verifier_instance(instances, builder, structure);
        }
        return instances;
    }

    static std::tuple<std::shared_ptr<ProverInstance>, std::shared_ptr<VerifierInstance>> fold_and_verify(
        const std::vector<std::shared_ptr<ProverInstance>>& prover_instances,
        const std::vector<std::shared_ptr<VerifierInstance>>& verifier_instances)
    {
        FoldingProver folding_prover(prover_instances);
        FoldingVerifier folding_verifier(verifier_instances);

        auto [prover_accumulator, folding_proof] = folding_prover.fold_instances();
        auto verifier_accumulator = folding_verifier.verify_folding_proof(folding_proof);
        return { prover_accumulator, verifier_accumulator };
    }

    static void check_accumulator_target_sum_manual(std::shared_ptr<ProverInstance>& accumulator, bool expected_result)
    {
        auto instance_size = accumulator->proving_key.circuit_size;
        auto expected_honk_evals = ProtoGalaxyProver::compute_full_honk_evaluations(
            accumulator->proving_key.polynomials, accumulator->alphas, accumulator->relation_parameters);
        // Construct pow(\vec{betas*}) as in the paper
        auto expected_pows = PowPolynomial(accumulator->gate_challenges);
        expected_pows.compute_values();

        // Compute the corresponding target sum and create a dummy accumulator
        auto expected_target_sum = FF(0);
        for (size_t i = 0; i < instance_size; i++) {
            expected_target_sum += expected_honk_evals[i] * expected_pows[i];
        }
        EXPECT_EQ(accumulator->target_sum == expected_target_sum, expected_result);
    }

    static void decide_and_verify(std::shared_ptr<ProverInstance>& prover_accumulator,
                                  std::shared_ptr<VerifierInstance>& verifier_accumulator,
                                  bool expected_result)
    {
        DeciderProver decider_prover(prover_accumulator);
        DeciderVerifier decider_verifier(verifier_accumulator);
        HonkProof decider_proof = decider_prover.construct_proof();
        bool verified = decider_verifier.verify_proof(decider_proof);
        EXPECT_EQ(verified, expected_result);
    }

    /**
     * @brief For a valid circuit, ensures that computing the value of the full UH/MegaHonk relation at each row in its
     * execution trace (with the contribution of the linearly dependent one added tot he first row, in case of
     * Goblin) will be 0.
     *
     */
    static void test_full_honk_evaluations_valid_circuit()
    {
        auto builder = typename Flavor::CircuitBuilder();
        construct_circuit(builder);

        auto instance = std::make_shared<ProverInstance>(builder);

        instance->relation_parameters.eta = FF::random_element();
        instance->relation_parameters.eta_two = FF::random_element();
        instance->relation_parameters.eta_three = FF::random_element();
        instance->relation_parameters.beta = FF::random_element();
        instance->relation_parameters.gamma = FF::random_element();

        instance->proving_key.add_ram_rom_memory_records_to_wire_4(instance->relation_parameters.eta,
                                                                   instance->relation_parameters.eta_two,
                                                                   instance->relation_parameters.eta_three);
        instance->proving_key.compute_logderivative_inverses(instance->relation_parameters);
        instance->proving_key.compute_grand_product_polynomials(instance->relation_parameters);

        for (auto& alpha : instance->alphas) {
            alpha = FF::random_element();
        }
        auto full_honk_evals = ProtoGalaxyProver::compute_full_honk_evaluations(
            instance->proving_key.polynomials, instance->alphas, instance->relation_parameters);

        // Evaluations should be 0 for valid circuit
        for (const auto& eval : full_honk_evals) {
            EXPECT_EQ(eval, FF(0));
        }
    }

    /**
     * @brief Check the coefficients of the perturbator computed from dummy \vec{β}, \vec{δ} and f_i(ω) will be the
     * same as if computed manually.
     *
     */
    static void test_pertubator_coefficients()
    {
        std::vector<FF> betas = { FF(5), FF(8), FF(11) };
        std::vector<FF> deltas = { FF(2), FF(4), FF(8) };
        std::vector<FF> full_honk_evaluations = { FF(1), FF(1), FF(1), FF(1), FF(1), FF(1), FF(1), FF(1) };
        auto perturbator = ProtoGalaxyProver::construct_perturbator_coefficients(betas, deltas, full_honk_evaluations);
        std::vector<FF> expected_values = { FF(648), FF(936), FF(432), FF(64) };
        EXPECT_EQ(perturbator.size(), 4); // log(instance_size) + 1
        for (size_t i = 0; i < perturbator.size(); i++) {
            EXPECT_EQ(perturbator[i], expected_values[i]);
        }
    }

    /**
     * @brief Create a dummy accumulator and ensure coefficient 0 of the computed perturbator is the same as the
     * accumulator's target sum.
     *
     */
    static void test_pertubator_polynomial()
    {
        using RelationSeparator = typename Flavor::RelationSeparator;
        const size_t log_instance_size(3);
        const size_t instance_size(1 << log_instance_size);
        // Construct fully random prover polynomials
        ProverPolynomials full_polynomials;
        for (auto& poly : full_polynomials.get_all()) {
            poly = bb::Polynomial<FF>::random(instance_size);
        }

        auto relation_parameters = bb::RelationParameters<FF>::get_random();
        RelationSeparator alphas;
        for (auto& alpha : alphas) {
            alpha = FF::random_element();
        }

        auto full_honk_evals =
            ProtoGalaxyProver::compute_full_honk_evaluations(full_polynomials, alphas, relation_parameters);
        std::vector<FF> betas(log_instance_size);
        for (size_t idx = 0; idx < log_instance_size; idx++) {
            betas[idx] = FF::random_element();
        }

        // Construct pow(\vec{betas}) as in the paper
        auto pow_beta = bb::PowPolynomial(betas);
        pow_beta.compute_values();

        // Compute the corresponding target sum and create a dummy accumulator
        auto target_sum = FF(0);
        for (size_t i = 0; i < instance_size; i++) {
            target_sum += full_honk_evals[i] * pow_beta[i];
        }

        auto accumulator = std::make_shared<ProverInstance>();
        accumulator->proving_key.polynomials = std::move(full_polynomials);
        accumulator->gate_challenges = betas;
        accumulator->target_sum = target_sum;
        accumulator->relation_parameters = relation_parameters;
        accumulator->alphas = alphas;

        auto deltas = ProtoGalaxyProver::compute_round_challenge_pows(log_instance_size, FF::random_element());
        auto perturbator = ProtoGalaxyProver::compute_perturbator(accumulator, deltas);

        // Ensure the constant coefficient of the perturbator is equal to the target sum as indicated by the paper
        EXPECT_EQ(perturbator[0], target_sum);
    }

    /**
     * @brief Manually compute the expected evaluations of the combiner quotient, given evaluations of the combiner
     * and check them against the evaluations returned by the function.
     *
     */
    static void test_combiner_quotient()
    {
        auto compressed_perturbator = FF(2); // F(\alpha) in the paper
        auto combiner = bb::Univariate<FF, 12>(std::array<FF, 12>{ 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31 });
        auto combiner_quotient = ProtoGalaxyProver::compute_combiner_quotient(compressed_perturbator, combiner);

        // K(i) = (G(i) - ( L_0(i) * F(\alpha)) / Z(i), i = {2,.., 13} for ProverInstances::NUM = 2
        // K(i) = (G(i) - (1 - i) * F(\alpha)) / i * (i - 1)
        auto expected_evals = bb::Univariate<FF, 12, 2>(std::array<FF, 10>{
            (FF(22) - (FF(1) - FF(2)) * compressed_perturbator) / (FF(2) * FF(2 - 1)),
            (FF(23) - (FF(1) - FF(3)) * compressed_perturbator) / (FF(3) * FF(3 - 1)),
            (FF(24) - (FF(1) - FF(4)) * compressed_perturbator) / (FF(4) * FF(4 - 1)),
            (FF(25) - (FF(1) - FF(5)) * compressed_perturbator) / (FF(5) * FF(5 - 1)),
            (FF(26) - (FF(1) - FF(6)) * compressed_perturbator) / (FF(6) * FF(6 - 1)),
            (FF(27) - (FF(1) - FF(7)) * compressed_perturbator) / (FF(7) * FF(7 - 1)),
            (FF(28) - (FF(1) - FF(8)) * compressed_perturbator) / (FF(8) * FF(8 - 1)),
            (FF(29) - (FF(1) - FF(9)) * compressed_perturbator) / (FF(9) * FF(9 - 1)),
            (FF(30) - (FF(1) - FF(10)) * compressed_perturbator) / (FF(10) * FF(10 - 1)),
            (FF(31) - (FF(1) - FF(11)) * compressed_perturbator) / (FF(11) * FF(11 - 1)),
        });

        for (size_t idx = 2; idx < 7; idx++) {
            EXPECT_EQ(combiner_quotient.value_at(idx), expected_evals.value_at(idx));
        }
    }

    /**
     * @brief For two dummy instances with their relation parameter η set, check that combining them in a
     * univariate, barycentrially extended to the desired number of evaluations, is performed correctly.
     *
     */
    static void test_combine_relation_parameters()
    {
        Builder builder1;
        auto instance1 = std::make_shared<ProverInstance>(builder1);
        instance1->relation_parameters.eta = 1;

        Builder builder2;
        builder2.add_variable(3);
        auto instance2 = std::make_shared<ProverInstance>(builder2);
        instance2->relation_parameters.eta = 3;

        ProverInstances instances{ { instance1, instance2 } };
        ProtoGalaxyProver::combine_relation_parameters(instances);

        bb::Univariate<FF, 11> expected_eta{ { 1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21 } };
        EXPECT_EQ(instances.relation_parameters.eta, expected_eta);
        // Optimised relation parameters are the same, we just don't compute any values for non-used indices when
        // deriving values from them
        for (size_t i = 0; i < 11; i++) {
            EXPECT_EQ(instances.optimised_relation_parameters.eta.evaluations[i], expected_eta.evaluations[i]);
        }
    }

    /**
     * @brief Given two dummy instances with the batching challenges alphas set (one for each subrelation) ensure
     * combining them in a univariate of desired length works as expected.
     */
    static void test_combine_alpha()
    {
        Builder builder1;
        auto instance1 = std::make_shared<ProverInstance>(builder1);
        instance1->alphas.fill(2);

        Builder builder2;
        builder2.add_variable(3);
        auto instance2 = std::make_shared<ProverInstance>(builder2);
        instance2->alphas.fill(4);

        ProverInstances instances{ { instance1, instance2 } };
        ProtoGalaxyProver::combine_alpha(instances);

        bb::Univariate<FF, 12> expected_alpha{ { 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24 } };
        for (const auto& alpha : instances.alphas) {
            EXPECT_EQ(alpha, expected_alpha);
        }
    }

    /**
     * @brief Testing one valid round of folding (plus decider) for two inhomogeneous circuits
     * @details For robustness we fold circuits with different numbers/types of gates (but the same dyadic size)
     *
     */
    static void test_protogalaxy_inhomogeneous()
    {
        auto check_fold_and_decide = [](Builder& circuit_1, Builder& circuit_2) {
            // Construct the prover/verifier instances for each
            TupleOfInstances instances;
            construct_prover_and_verifier_instance(instances, circuit_1);
            construct_prover_and_verifier_instance(instances, circuit_2);

            // Perform prover and verifier folding
            auto [prover_accumulator, verifier_accumulator] = fold_and_verify(get<0>(instances), get<1>(instances));
            check_accumulator_target_sum_manual(prover_accumulator, true);

            // Run decider
            decide_and_verify(prover_accumulator, verifier_accumulator, true);
        };

        // One circuit has more arithmetic gates
        {
            // Construct two equivalent circuits
            Builder builder1;
            Builder builder2;
            construct_circuit(builder1);
            construct_circuit(builder2);

            // Add some arithmetic gates
            bb::MockCircuits::add_arithmetic_gates(builder1, /*num_gates=*/4);

            check_fold_and_decide(builder1, builder2);
        }

        // One circuit has more arithmetic gates with public inputs
        {
            // Construct two equivalent circuits
            Builder builder1;
            Builder builder2;
            construct_circuit(builder1);
            construct_circuit(builder2);

            // Add some arithmetic gates with public inputs to the first circuit
            bb::MockCircuits::add_arithmetic_gates_with_public_inputs(builder1, /*num_gates=*/4);

            check_fold_and_decide(builder1, builder2);
        }

        // One circuit has more lookup gates
        {
            // Construct two equivalent circuits
            Builder builder1;
            Builder builder2;
            construct_circuit(builder1);
            construct_circuit(builder2);

            // Add a different number of lookup gates to each circuit
            bb::MockCircuits::add_lookup_gates(builder1, /*num_iterations=*/2); // 12 gates plus 4096 table
            bb::MockCircuits::add_lookup_gates(builder2, /*num_iterations=*/1); // 6 gates plus 4096 table

            check_fold_and_decide(builder1, builder2);
        }
    }

    /**
     * @brief Ensure failure for a bad lookup gate in one of the circuits being folded
     *
     */
    static void test_protogalaxy_bad_lookup_failure()
    {
        // Construct two equivalent circuits
        Builder builder1;
        Builder builder2;
        construct_circuit(builder1);
        construct_circuit(builder2);

        // Add a different number of lookup gates to each circuit
        bb::MockCircuits::add_lookup_gates(builder1, /*num_iterations=*/2); // 12 gates plus 4096 table
        bb::MockCircuits::add_lookup_gates(builder2, /*num_iterations=*/1); // 6 gates plus 4096 table

        // Erroneously set a non-zero wire value to zero in one of the lookup gates
        for (auto& wire_3_witness_idx : builder1.blocks.lookup.w_o()) {
            if (wire_3_witness_idx != builder1.zero_idx) {
                wire_3_witness_idx = builder1.zero_idx;
                break;
            }
        }

        // Construct the prover/verifier instances for each
        TupleOfInstances instances;
        construct_prover_and_verifier_instance(instances, builder1);
        construct_prover_and_verifier_instance(instances, builder2);

        // Perform prover and verifier folding
        auto [prover_accumulator, verifier_accumulator] = fold_and_verify(get<0>(instances), get<1>(instances));

        // Expect failure in manual target sum check and decider
        bool expected_result = false;
        check_accumulator_target_sum_manual(prover_accumulator, expected_result);
        decide_and_verify(prover_accumulator, verifier_accumulator, expected_result);
    }

    /**
     * @brief Testing two valid rounds of folding followed by the decider.
     *
     */
    static void test_full_protogalaxy()
    {
        TupleOfInstances insts = construct_instances(2);
        auto [prover_accumulator, verifier_accumulator] = fold_and_verify(get<0>(insts), get<1>(insts));
        check_accumulator_target_sum_manual(prover_accumulator, true);

        TupleOfInstances insts_2 = construct_instances(1); // just one set of prover/verifier instances
        auto [prover_accumulator_2, verifier_accumulator_2] =
            fold_and_verify({ prover_accumulator, get<0>(insts_2)[0] }, { verifier_accumulator, get<1>(insts_2)[0] });
        check_accumulator_target_sum_manual(prover_accumulator_2, true);

        decide_and_verify(prover_accumulator_2, verifier_accumulator_2, true);
    }

    /**
     * @brief Testing two valid rounds of folding followed by the decider for a structured trace.
     *
     */
    static void test_full_protogalaxy_structured_trace()
    {
        TraceStructure trace_structure = TraceStructure::SMALL_TEST;
        TupleOfInstances instances = construct_instances(2, trace_structure);

        auto [prover_accumulator, verifier_accumulator] = fold_and_verify(get<0>(instances), get<1>(instances));
        check_accumulator_target_sum_manual(prover_accumulator, true);

        TupleOfInstances instances_2 =
            construct_instances(1, trace_structure); // just one set of prover/verifier instances

        auto [prover_accumulator_2, verifier_accumulator_2] = fold_and_verify(
            { prover_accumulator, get<0>(instances_2)[0] }, { verifier_accumulator, get<1>(instances_2)[0] });
        check_accumulator_target_sum_manual(prover_accumulator_2, true);
        info(prover_accumulator_2->proving_key.circuit_size);
        decide_and_verify(prover_accumulator_2, verifier_accumulator_2, true);
    }

    /**
     * @brief Testing two valid rounds of folding followed by the decider for a structured trace.
     * @details Here we're interested in folding inhomogeneous circuits, i.e. circuits with different numbers of
     * constraints, which should be automatically handled by the structured trace
     *
     */
    static void test_full_protogalaxy_structured_trace_inhomogeneous_circuits()
    {
        TraceStructure trace_structure = TraceStructure::SMALL_TEST;

        // Construct three circuits to be folded, each with a different number of constraints
        Builder builder1;
        Builder builder2;
        Builder builder3;
        construct_circuit(builder1);
        construct_circuit(builder2);
        construct_circuit(builder3);

        // Create inhomogenous circuits by adding a different number of add gates to each
        MockCircuits::add_arithmetic_gates(builder1, 10);
        MockCircuits::add_arithmetic_gates(builder2, 100);
        MockCircuits::add_arithmetic_gates(builder3, 1000);

        // Construct the Prover/Verifier instances for the first two circuits
        TupleOfInstances instances;
        construct_prover_and_verifier_instance(instances, builder1, trace_structure);
        construct_prover_and_verifier_instance(instances, builder2, trace_structure);

        // Fold the first two instances
        auto [prover_accumulator, verifier_accumulator] = fold_and_verify(get<0>(instances), get<1>(instances));
        check_accumulator_target_sum_manual(prover_accumulator, true);

        // Construct the Prover/Verifier instance for the third circuit
        TupleOfInstances instances_2;
        construct_prover_and_verifier_instance(instances_2, builder3, trace_structure);

        // Fold 3rd instance into accumulator
        auto [prover_accumulator_2, verifier_accumulator_2] = fold_and_verify(
            { prover_accumulator, get<0>(instances_2)[0] }, { verifier_accumulator, get<1>(instances_2)[0] });
        check_accumulator_target_sum_manual(prover_accumulator_2, true);
        info(prover_accumulator_2->proving_key.circuit_size);

        // Decide on final accumulator
        decide_and_verify(prover_accumulator_2, verifier_accumulator_2, true);
    }

    /**
     * @brief Ensure tampering a commitment and then calling the decider causes the decider verification to fail.
     *
     */
    static void test_tampered_commitment()
    {
        TupleOfInstances insts = construct_instances(2);
        auto [prover_accumulator, verifier_accumulator] = fold_and_verify(get<0>(insts), get<1>(insts));
        check_accumulator_target_sum_manual(prover_accumulator, true);

        // Tamper with a commitment
        verifier_accumulator->witness_commitments.w_l = Projective(Affine::random_element());

        TupleOfInstances insts_2 = construct_instances(1); // just one set of prover/verifier instances
        auto [prover_accumulator_2, verifier_accumulator_2] =
            fold_and_verify({ prover_accumulator, get<0>(insts_2)[0] }, { verifier_accumulator, get<1>(insts_2)[0] });
        check_accumulator_target_sum_manual(prover_accumulator_2, true);

        decide_and_verify(prover_accumulator_2, verifier_accumulator_2, false);
    }

    /**
     * @brief Ensure tampering an accumulator and then calling fold again causes the target sums in the prover and
     * verifier accumulators to be different and decider verification to fail.
     *
     */
    static void test_tampered_accumulator_polynomial()
    {
        TupleOfInstances insts = construct_instances(2);
        auto [prover_accumulator, verifier_accumulator] = fold_and_verify(get<0>(insts), get<1>(insts));
        check_accumulator_target_sum_manual(prover_accumulator, true);

        // Tamper with an accumulator polynomial
        prover_accumulator->proving_key.polynomials.w_l[1] = FF::random_element();
        check_accumulator_target_sum_manual(prover_accumulator, false);

        TupleOfInstances insts_2 = construct_instances(1); // just one set of prover/verifier instances
        auto [prover_accumulator_2, verifier_accumulator_2] =
            fold_and_verify({ prover_accumulator, get<0>(insts_2)[0] }, { verifier_accumulator, get<1>(insts_2)[0] });

        EXPECT_EQ(prover_accumulator_2->target_sum == verifier_accumulator_2->target_sum, false);
        decide_and_verify(prover_accumulator_2, verifier_accumulator_2, false);
    }

    template <size_t k> static void test_fold_k_instances()
    {
        constexpr size_t total_insts = k + 1;
        TupleOfInstances insts = construct_instances(total_insts);

        ProtoGalaxyProver_<ProverInstances_<Flavor, total_insts>> folding_prover(get<0>(insts));
        ProtoGalaxyVerifier_<VerifierInstances_<Flavor, total_insts>> folding_verifier(get<1>(insts));

        auto [prover_accumulator, folding_proof] = folding_prover.fold_instances();
        auto verifier_accumulator = folding_verifier.verify_folding_proof(folding_proof);
        check_accumulator_target_sum_manual(prover_accumulator, true);

        decide_and_verify(prover_accumulator, verifier_accumulator, true);
    }
};
} // namespace

using FlavorTypes = testing::Types<UltraFlavor, MegaFlavor>;
TYPED_TEST_SUITE(ProtoGalaxyTests, FlavorTypes);

TYPED_TEST(ProtoGalaxyTests, PerturbatorCoefficients)
{
    TestFixture::test_pertubator_coefficients();
}

TYPED_TEST(ProtoGalaxyTests, FullHonkEvaluationsValidCircuit)
{
    TestFixture::test_full_honk_evaluations_valid_circuit();
}

TYPED_TEST(ProtoGalaxyTests, PerturbatorPolynomial)
{
    TestFixture::test_pertubator_polynomial();
}

TYPED_TEST(ProtoGalaxyTests, CombinerQuotient)
{
    TestFixture::test_combiner_quotient();
}

TYPED_TEST(ProtoGalaxyTests, CombineRelationParameters)
{
    TestFixture::test_combine_relation_parameters();
}

TYPED_TEST(ProtoGalaxyTests, CombineAlpha)
{
    TestFixture::test_combine_alpha();
}

TYPED_TEST(ProtoGalaxyTests, ProtogalaxyInhomogeneous)
{
    TestFixture::test_protogalaxy_inhomogeneous();
}

TYPED_TEST(ProtoGalaxyTests, FullProtogalaxyTest)
{
    TestFixture::test_full_protogalaxy();
}

TYPED_TEST(ProtoGalaxyTests, FullProtogalaxyStructuredTrace)
{
    TestFixture::test_full_protogalaxy_structured_trace();
}
TYPED_TEST(ProtoGalaxyTests, FullProtogalaxyStructuredTraceInhomogeneous)
{
    TestFixture::test_full_protogalaxy_structured_trace_inhomogeneous_circuits();
}

TYPED_TEST(ProtoGalaxyTests, TamperedCommitment)
{
    TestFixture::test_tampered_commitment();
}

TYPED_TEST(ProtoGalaxyTests, TamperedAccumulatorPolynomial)
{
    TestFixture::test_tampered_accumulator_polynomial();
}

TYPED_TEST(ProtoGalaxyTests, BadLookupFailure)
{
    TestFixture::test_protogalaxy_bad_lookup_failure();
}

// We only fold one instance currently due to significant compile time added by multiple instances
TYPED_TEST(ProtoGalaxyTests, Fold1Instance)
{
    TestFixture::template test_fold_k_instances<1>();
}