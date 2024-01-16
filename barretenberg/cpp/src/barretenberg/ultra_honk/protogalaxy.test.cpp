#include "barretenberg/polynomials/pow.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"
#include <gtest/gtest.h>

using namespace proof_system::honk;

using Flavor = flavor::Ultra;
using VerificationKey = Flavor::VerificationKey;
using Instance = ProverInstance_<Flavor>;
using Instances = ProverInstances_<Flavor, 2>;
using ProtoGalaxyProver = ProtoGalaxyProver_<Instances>;
using FF = Flavor::FF;
using Affine = Flavor::Commitment;
using Projective = Flavor::GroupElement;
using Builder = Flavor::CircuitBuilder;
using Polynomial = typename Flavor::Polynomial;
using ProverPolynomials = Flavor::ProverPolynomials;
using RelationParameters = proof_system::RelationParameters<FF>;
using WitnessCommitments = typename Flavor::WitnessCommitments;
using CommitmentKey = Flavor::CommitmentKey;
using PowPolynomial = bb::PowPolynomial<FF>;

const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;

namespace bb::protogalaxy_tests {
namespace {
auto& engine = numeric::random::get_debug_engine();
}
// TODO(https://github.com/AztecProtocol/barretenberg/issues/744): make testing utility with functionality shared
// amongst test files in the proof system
bb::Polynomial<FF> get_random_polynomial(size_t size)
{
    auto poly = bb::Polynomial<FF>(size);
    for (auto& coeff : poly) {
        coeff = FF::random_element();
    }
    return poly;
}

ProverPolynomials construct_ultra_full_polynomials(auto& input_polynomials)
{
    ProverPolynomials full_polynomials;
    for (auto [prover_poly, input_poly] : zip_view(full_polynomials.get_all(), input_polynomials)) {
        prover_poly = input_poly.share();
    }
    return full_polynomials;
}

std::shared_ptr<Instance> fold_and_verify(const std::vector<std::shared_ptr<Instance>>& instances,
                                          UltraComposer& composer,
                                          bool expected_result)
{
    auto folding_prover = composer.create_folding_prover(instances, composer.commitment_key);
    auto folding_verifier = composer.create_folding_verifier();

    auto proof = folding_prover.fold_instances();
    auto next_accumulator = proof.accumulator;
    auto res = folding_verifier.verify_folding_proof(proof.folding_data);
    EXPECT_EQ(res, expected_result);
    return next_accumulator;
}

void check_accumulator_target_sum_manual(std::shared_ptr<Instance>& accumulator, bool expected_result)
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
void decide_and_verify(std::shared_ptr<Instance>& accumulator, UltraComposer& composer, bool expected_result)
{
    auto decider_prover = composer.create_decider_prover(accumulator, composer.commitment_key);
    auto decider_verifier = composer.create_decider_verifier(accumulator);
    auto decision = decider_prover.construct_proof();
    auto verified = decider_verifier.verify_proof(decision);
    EXPECT_EQ(verified, expected_result);
}

class ProtoGalaxyTests : public ::testing::Test {
  public:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};

TEST_F(ProtoGalaxyTests, FullHonkEvaluationsValidCircuit)
{
    auto builder = Builder();
    FF a = FF::one();
    uint32_t a_idx = builder.add_public_variable(a);
    FF b = FF::one();
    FF c = a + b;
    uint32_t b_idx = builder.add_variable(b);
    uint32_t c_idx = builder.add_variable(c);
    builder.create_add_gate({ a_idx, b_idx, c_idx, 1, 1, -1, 0 });
    builder.create_add_gate({ a_idx, b_idx, c_idx, 1, 1, -1, 0 });

    auto composer = UltraComposer();
    auto instance = composer.create_instance(builder);
    instance->initialize_prover_polynomials();

    auto eta = FF::random_element();
    auto beta = FF::random_element();
    auto gamma = FF::random_element();
    instance->compute_sorted_accumulator_polynomials(eta);
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
TEST_F(ProtoGalaxyTests, PerturbatorCoefficients)
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

TEST_F(ProtoGalaxyTests, PerturbatorPolynomial)
{
    using RelationSeparator = Flavor::RelationSeparator;
    const size_t log_instance_size(3);
    const size_t instance_size(1 << log_instance_size);

    std::array<bb::Polynomial<FF>, NUM_POLYNOMIALS> random_polynomials;
    for (auto& poly : random_polynomials) {
        poly = get_random_polynomial(instance_size);
    }
    auto full_polynomials = construct_ultra_full_polynomials(random_polynomials);
    auto relation_parameters = proof_system::RelationParameters<FF>::get_random();
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
    auto pow_beta = PowPolynomial(betas);
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

TEST_F(ProtoGalaxyTests, CombinerQuotient)
{
    auto compressed_perturbator = FF(2); // F(\alpha) in the paper
    auto combiner = bb::Univariate<FF, 13>(std::array<FF, 13>{ 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32 });
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

TEST_F(ProtoGalaxyTests, CombineRelationParameters)
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

    Univariate<FF, 12> expected_eta{ { 1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23 } };
    EXPECT_EQ(instances.relation_parameters.eta, expected_eta);
}

TEST_F(ProtoGalaxyTests, CombineAlpha)
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

    Univariate<FF, 13> expected_alpha{ { 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26 } };
    for (const auto& alpha : instances.alphas) {
        EXPECT_EQ(alpha, expected_alpha);
    }
}

// Check both manually and using the protocol two rounds of folding
TEST_F(ProtoGalaxyTests, FullProtogalaxyTest)
{
    auto composer = UltraComposer();

    auto builder_1 = typename Flavor::CircuitBuilder();
    builder_1.add_public_variable(FF(1));

    auto instance_1 = composer.create_instance(builder_1);

    auto builder_2 = typename Flavor::CircuitBuilder();
    builder_2.add_public_variable(FF(1));

    auto instance_2 = composer.create_instance(builder_2);

    auto instances = std::vector<std::shared_ptr<Instance>>{ instance_1, instance_2 };
    auto first_accumulator = fold_and_verify(instances, composer, true);
    check_accumulator_target_sum_manual(first_accumulator, true);

    auto builder_3 = typename Flavor::CircuitBuilder();
    builder_3.add_public_variable(FF(1));
    auto instance_3 = composer.create_instance(builder_3);

    instances = std::vector<std::shared_ptr<Instance>>{ first_accumulator, instance_3 };
    auto second_accumulator = fold_and_verify(instances, composer, true);
    check_accumulator_target_sum_manual(second_accumulator, true);

    decide_and_verify(second_accumulator, composer, true);
}

TEST_F(ProtoGalaxyTests, TamperedCommitment)
{
    auto composer = UltraComposer();

    auto builder_1 = typename Flavor::CircuitBuilder();
    builder_1.add_public_variable(FF(1));

    auto instance_1 = composer.create_instance(builder_1);

    auto builder_2 = typename Flavor::CircuitBuilder();
    builder_2.add_public_variable(FF(1));

    auto instance_2 = composer.create_instance(builder_2);

    auto instances = std::vector<std::shared_ptr<Instance>>{ instance_1, instance_2 };
    auto first_accumulator = fold_and_verify(instances, composer, true);
    check_accumulator_target_sum_manual(first_accumulator, true);

    auto builder_3 = typename Flavor::CircuitBuilder();
    builder_3.add_public_variable(FF(1));
    auto instance_3 = composer.create_instance(builder_3);

    // tampering with the commitment should cause the decider to fail
    first_accumulator->witness_commitments.w_l = Projective(Affine::random_element());
    instances = std::vector<std::shared_ptr<Instance>>{ first_accumulator, instance_3 };

    auto second_accumulator = fold_and_verify(instances, composer, true);

    decide_and_verify(second_accumulator, composer, false);
}

TEST_F(ProtoGalaxyTests, TamperedAccumulatorPolynomial)
{
    auto composer = UltraComposer();

    auto builder_1 = typename Flavor::CircuitBuilder();
    builder_1.add_public_variable(FF(1));

    auto instance_1 = composer.create_instance(builder_1);

    auto builder_2 = typename Flavor::CircuitBuilder();
    builder_2.add_public_variable(FF(1));

    auto instance_2 = composer.create_instance(builder_2);

    auto instances = std::vector<std::shared_ptr<Instance>>{ instance_1, instance_2 };
    auto first_accumulator = fold_and_verify(instances, composer, true);
    check_accumulator_target_sum_manual(first_accumulator, true);

    auto builder_3 = typename Flavor::CircuitBuilder();
    builder_3.add_public_variable(FF(1));
    auto instance_3 = composer.create_instance(builder_3);

    // tampering with accumulator's polynomial should cause both folding and deciding to fail
    instances = std::vector<std::shared_ptr<Instance>>{ first_accumulator, instance_3 };
    first_accumulator->prover_polynomials.w_l[1] = FF::random_element();
    auto second_accumulator = fold_and_verify(instances, composer, false);

    decide_and_verify(second_accumulator, composer, false);
}

} // namespace bb::protogalaxy_tests