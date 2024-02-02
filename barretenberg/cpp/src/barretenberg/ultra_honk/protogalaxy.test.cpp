#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/polynomials/pow.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"
#include <gtest/gtest.h>

using namespace bb;

namespace {

auto& engine = numeric::get_debug_randomness();

template <typename Flavor> class ProtoGalaxyTests : public testing::Test {
  public:
    using Composer = UltraComposer_<Flavor>;
    using VerificationKey = typename Flavor::VerificationKey;
    using Instance = ProverInstance_<Flavor>;
    using Instances = ProverInstances_<Flavor, 2>;
    using ProtoGalaxyProver = ProtoGalaxyProver_<Instances>;
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

    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }

    static void construct_circuit(Builder& builder)
    {
        if constexpr (IsGoblinFlavor<Flavor>) {
            GoblinMockCircuits::construct_arithmetic_circuit(builder);
            GoblinMockCircuits::construct_goblin_ecc_op_circuit(builder);

        } else {
            FF a = FF::random_element();
            FF b = FF::random_element();
            FF c = FF::random_element();
            FF d = a + b + c;
            uint32_t a_idx = builder.add_public_variable(a);
            uint32_t b_idx = builder.add_variable(b);
            uint32_t c_idx = builder.add_variable(c);
            uint32_t d_idx = builder.add_variable(d);

            builder.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, FF(1), FF(1), FF(1), FF(-1), FF(0) });
        }
    }

    static ProverPolynomials construct_full_prover_polynomials(auto& input_polynomials)
    {
        ProverPolynomials full_polynomials;
        for (auto [prover_poly, input_poly] : zip_view(full_polynomials.get_all(), input_polynomials)) {
            prover_poly = input_poly.share();
        }
        return full_polynomials;
    }

    static std::shared_ptr<Instance> fold_and_verify(const std::vector<std::shared_ptr<Instance>>& instances,
                                                     Composer& composer,
                                                     bool expected_result)
    {
        auto folding_prover = composer.create_folding_prover(instances);
        auto folding_verifier = composer.create_folding_verifier();

        auto proof = folding_prover.fold_instances();
        auto next_accumulator = proof.accumulator;
        auto res = folding_verifier.verify_folding_proof(proof.folding_data);
        EXPECT_EQ(res, expected_result);
        return next_accumulator;
    }

    static void check_accumulator_target_sum_manual(std::shared_ptr<Instance>& accumulator, bool expected_result)
    {
        auto instance_size = accumulator->instance_size;
        auto expected_honk_evals = ProtoGalaxyProver::compute_full_honk_evaluations(
            accumulator->prover_polynomials, accumulator->alphas, accumulator->relation_parameters);
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

    static void decide_and_verify(std::shared_ptr<Instance>& accumulator, Composer& composer, bool expected_result)
    {
        auto decider_prover = composer.create_decider_prover(accumulator);
        auto decider_verifier = composer.create_decider_verifier(accumulator);
        auto decider_proof = decider_prover.construct_proof();
        auto verified = decider_verifier.verify_proof(decider_proof);
        EXPECT_EQ(verified, expected_result);
    }

    /**
     * @brief For a valid circuit, ensures that computing the value of the full UH/UGH relation at each row in its
     * execution trace (with the contribution of the linearly dependent one added tot he first row, in case of Goblin)
     * will be 0.
     *
     */
    static void test_full_honk_evaluations_valid_circuit()
    {
        auto builder = typename Flavor::CircuitBuilder();
        construct_circuit(builder);

        auto composer = Composer();
        auto instance = composer.create_instance(builder);
        instance->initialize_prover_polynomials();

        auto eta = FF::random_element();
        auto beta = FF::random_element();
        auto gamma = FF::random_element();
        instance->compute_sorted_accumulator_polynomials(eta);
        if constexpr (IsGoblinFlavor<Flavor>) {
            instance->compute_logderivative_inverse(beta, gamma);
        }
        instance->compute_grand_product_polynomials(beta, gamma);

        for (auto& alpha : instance->alphas) {
            alpha = FF::random_element();
        }
        auto full_honk_evals = ProtoGalaxyProver::compute_full_honk_evaluations(
            instance->prover_polynomials, instance->alphas, instance->relation_parameters);

        // Evaluations should be 0 for valid circuit
        for (const auto& eval : full_honk_evals) {
            EXPECT_EQ(eval, FF(0));
        }
    }

    /**
     * @brief Check the coefficients of the perturbator computed from dummy \vec{β}, \vec{δ} and f_i(ω) will be the same
     * as if computed manually.
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
        std::array<bb::Polynomial<FF>, Flavor::NUM_ALL_ENTITIES> random_polynomials;
        for (auto& poly : random_polynomials) {
            poly = bb::Polynomial<FF>::random(instance_size);
        }
        auto full_polynomials = construct_full_prover_polynomials(random_polynomials);
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

        auto accumulator = std::make_shared<Instance>();
        accumulator->prover_polynomials = std::move(full_polynomials);
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
     * @brief Manually compute the expected evaluations of the combiner quotient, given evaluations of the combiner and
     * check them against the evaluations returned by the function.
     *
     */
    static void test_combiner_quotient()
    {
        auto compressed_perturbator = FF(2); // F(\alpha) in the paper
        auto combiner =
            bb::Univariate<FF, 13>(std::array<FF, 13>{ 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32 });
        auto combiner_quotient = ProtoGalaxyProver::compute_combiner_quotient(compressed_perturbator, combiner);

        // K(i) = (G(i) - ( L_0(i) * F(\alpha)) / Z(i), i = {2,.., 13} for ProverInstances::NUM = 2
        // K(i) = (G(i) - (1 - i) * F(\alpha)) / i * (i - 1)
        auto expected_evals = bb::Univariate<FF, 13, 2>(std::array<FF, 11>{
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
            (FF(32) - (FF(1) - FF(12)) * compressed_perturbator) / (FF(12) * FF(12 - 1)),
        });

        for (size_t idx = 2; idx < 7; idx++) {
            EXPECT_EQ(combiner_quotient.value_at(idx), expected_evals.value_at(idx));
        }
    }

    /**
     * @brief For two dummy instances with their relation parameter η set, check that combining them in a univariate,
     * barycentrially extended to the desired number of evaluations, is performed correctly.
     *
     */
    static void test_combine_relation_parameters()
    {
        using Instances = ProverInstances_<Flavor, 2>;
        using Instance = typename Instances::Instance;

        Builder builder1;
        auto instance1 = std::make_shared<Instance>(builder1);
        instance1->relation_parameters.eta = 1;

        Builder builder2;
        builder2.add_variable(3);
        auto instance2 = std::make_shared<Instance>(builder2);
        instance2->relation_parameters.eta = 3;

        Instances instances{ { instance1, instance2 } };
        ProtoGalaxyProver::combine_relation_parameters(instances);

        bb::Univariate<FF, 12> expected_eta{ { 1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23 } };
        EXPECT_EQ(instances.relation_parameters.eta, expected_eta);
    }

    /**
     * @brief Given two dummy instances with the batching challenges alphas set (one for each subrelation) ensure
     * combining them in a univariate of desired length works as expected.
     */
    static void test_combine_alpha()
    {
        using Instances = ProverInstances_<Flavor, 2>;
        using Instance = typename Instances::Instance;

        Builder builder1;
        auto instance1 = std::make_shared<Instance>(builder1);
        instance1->alphas.fill(2);

        Builder builder2;
        builder2.add_variable(3);
        auto instance2 = std::make_shared<Instance>(builder2);
        instance2->alphas.fill(4);

        Instances instances{ { instance1, instance2 } };
        ProtoGalaxyProver::combine_alpha(instances);

        bb::Univariate<FF, 13> expected_alpha{ { 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26 } };
        for (const auto& alpha : instances.alphas) {
            EXPECT_EQ(alpha, expected_alpha);
        }
    }

    /**
     * @brief Testing two valid rounds of folding followed by the decider.
     *
     */
    static void test_full_protogalaxy()
    {
        auto composer = Composer();
        auto builder_1 = typename Flavor::CircuitBuilder();
        construct_circuit(builder_1);

        auto instance_1 = composer.create_instance(builder_1);

        auto builder_2 = typename Flavor::CircuitBuilder();
        construct_circuit(builder_2);

        auto instance_2 = composer.create_instance(builder_2);

        auto instances = std::vector<std::shared_ptr<Instance>>{ instance_1, instance_2 };
        auto first_accumulator = fold_and_verify(instances, composer, true);
        check_accumulator_target_sum_manual(first_accumulator, true);

        auto builder_3 = typename Flavor::CircuitBuilder();
        construct_circuit(builder_3);
        auto instance_3 = composer.create_instance(builder_3);

        instances = std::vector<std::shared_ptr<Instance>>{ first_accumulator, instance_3 };
        auto second_accumulator = fold_and_verify(instances, composer, true);
        check_accumulator_target_sum_manual(second_accumulator, true);

        decide_and_verify(first_accumulator, composer, true);
    }

    /**
     * @brief Ensure tampering a commitment and then calling the decider causes the decider verification to fail.
     *
     */
    static void test_tampered_commitment()
    {
        auto composer = Composer();

        auto builder_1 = typename Flavor::CircuitBuilder();
        construct_circuit(builder_1);

        auto instance_1 = composer.create_instance(builder_1);

        auto builder_2 = typename Flavor::CircuitBuilder();
        construct_circuit(builder_2);

        auto instance_2 = composer.create_instance(builder_2);

        auto instances = std::vector<std::shared_ptr<Instance>>{ instance_1, instance_2 };
        auto first_accumulator = fold_and_verify(instances, composer, true);
        check_accumulator_target_sum_manual(first_accumulator, true);

        auto builder_3 = typename Flavor::CircuitBuilder();
        construct_circuit(builder_3);
        auto instance_3 = composer.create_instance(builder_3);

        // tampering with the commitment should cause the decider to fail
        first_accumulator->witness_commitments.w_l = Projective(Affine::random_element());
        instances = std::vector<std::shared_ptr<Instance>>{ first_accumulator, instance_3 };

        auto second_accumulator = fold_and_verify(instances, composer, true);

        decide_and_verify(second_accumulator, composer, false);
    }

    /**
     * @brief Ensure tampering an accumulator and then calling fold again causes both the folding verification and
     * decider verification to fail.
     *
     */
    static void test_tampered_accumulator_polynomial()
    {
        auto composer = Composer();

        auto builder_1 = typename Flavor::CircuitBuilder();
        construct_circuit(builder_1);

        auto instance_1 = composer.create_instance(builder_1);

        auto builder_2 = typename Flavor::CircuitBuilder();
        construct_circuit(builder_2);

        auto instance_2 = composer.create_instance(builder_2);

        auto instances = std::vector<std::shared_ptr<Instance>>{ instance_1, instance_2 };
        auto first_accumulator = fold_and_verify(instances, composer, true);
        check_accumulator_target_sum_manual(first_accumulator, true);

        auto builder_3 = typename Flavor::CircuitBuilder();
        construct_circuit(builder_3);
        auto instance_3 = composer.create_instance(builder_3);

        // tampering with accumulator's polynomial should cause both folding and deciding to fail
        instances = std::vector<std::shared_ptr<Instance>>{ first_accumulator, instance_3 };
        first_accumulator->prover_polynomials.w_l[1] = FF::random_element();
        auto second_accumulator = fold_and_verify(instances, composer, false);

        decide_and_verify(second_accumulator, composer, false);
    }
};
} // namespace

using FlavorTypes = testing::Types<UltraFlavor, GoblinUltraFlavor>;
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

TYPED_TEST(ProtoGalaxyTests, FullProtogalaxyTest)
{
    TestFixture::test_full_protogalaxy();
}

TYPED_TEST(ProtoGalaxyTests, TamperedCommitment)
{
    TestFixture::test_tampered_commitment();
}

TYPED_TEST(ProtoGalaxyTests, TamperedAccumulatorPolynomial)
{
    TestFixture::test_tampered_accumulator_polynomial();
}