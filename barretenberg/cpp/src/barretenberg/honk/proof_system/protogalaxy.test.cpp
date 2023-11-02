#include "barretenberg/honk/composer/ultra_composer.hpp"
#include "protogalaxy_prover.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace proof_system::honk;

using Flavor = flavor::Ultra;
using Instance = ProverInstance_<Flavor>;
using Instances = ProverInstances_<Flavor, 2>;
using ProtoGalaxyProver = ProtoGalaxyProver_<Instances>;
using FF = Flavor::FF;
using Builder = Flavor::CircuitBuilder;
using ProverPolynomials = Flavor::ProverPolynomials;
const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;

namespace protogalaxy_tests {
namespace {
auto& engine = numeric::random::get_debug_engine();
}
// TODO(https://github.com/AztecProtocol/barretenberg/issues/744): make testing utility with functionality shared
// amongst test files in the proof system
barretenberg::Polynomial<FF> get_random_polynomial(size_t size)
{
    auto poly = barretenberg::Polynomial<FF>(size);
    for (auto& coeff : poly) {
        coeff = FF::random_element();
    }
    return poly;
}

ProverPolynomials construct_ultra_full_polynomials(auto& input_polynomials)
{
    ProverPolynomials full_polynomials;
    full_polynomials.q_c = input_polynomials[0];
    full_polynomials.q_l = input_polynomials[1];
    full_polynomials.q_r = input_polynomials[2];
    full_polynomials.q_o = input_polynomials[3];
    full_polynomials.q_4 = input_polynomials[4];
    full_polynomials.q_m = input_polynomials[5];
    full_polynomials.q_arith = input_polynomials[6];
    full_polynomials.q_sort = input_polynomials[7];
    full_polynomials.q_elliptic = input_polynomials[8];
    full_polynomials.q_aux = input_polynomials[9];
    full_polynomials.q_lookup = input_polynomials[10];
    full_polynomials.sigma_1 = input_polynomials[11];
    full_polynomials.sigma_2 = input_polynomials[12];
    full_polynomials.sigma_3 = input_polynomials[13];
    full_polynomials.sigma_4 = input_polynomials[14];
    full_polynomials.id_1 = input_polynomials[15];
    full_polynomials.id_2 = input_polynomials[16];
    full_polynomials.id_3 = input_polynomials[17];
    full_polynomials.id_4 = input_polynomials[18];
    full_polynomials.table_1 = input_polynomials[19];
    full_polynomials.table_2 = input_polynomials[20];
    full_polynomials.table_3 = input_polynomials[21];
    full_polynomials.table_4 = input_polynomials[22];
    full_polynomials.lagrange_first = input_polynomials[23];
    full_polynomials.lagrange_last = input_polynomials[24];
    full_polynomials.w_l = input_polynomials[25];
    full_polynomials.w_r = input_polynomials[26];
    full_polynomials.w_o = input_polynomials[27];
    full_polynomials.w_4 = input_polynomials[28];
    full_polynomials.sorted_accum = input_polynomials[29];
    full_polynomials.z_perm = input_polynomials[30];
    full_polynomials.z_lookup = input_polynomials[31];
    full_polynomials.table_1_shift = input_polynomials[32];
    full_polynomials.table_2_shift = input_polynomials[33];
    full_polynomials.table_3_shift = input_polynomials[34];
    full_polynomials.table_4_shift = input_polynomials[35];
    full_polynomials.w_l_shift = input_polynomials[36];
    full_polynomials.w_r_shift = input_polynomials[37];
    full_polynomials.w_o_shift = input_polynomials[38];
    full_polynomials.w_4_shift = input_polynomials[39];
    full_polynomials.sorted_accum_shift = input_polynomials[40];
    full_polynomials.z_perm_shift = input_polynomials[41];
    full_polynomials.z_lookup_shift = input_polynomials[42];

    return full_polynomials;
}

class ProtoGalaxyTests : public ::testing::Test {
  public:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../srs_db/ignition"); }
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
    instance->initialise_prover_polynomials();

    auto eta = FF::random_element();
    auto beta = FF::random_element();
    auto gamma = FF::random_element();
    instance->compute_sorted_accumulator_polynomials(eta);
    instance->compute_grand_product_polynomials(beta, gamma);

    auto alpha = FF::random_element();
    auto full_honk_evals = ProtoGalaxyProver::compute_full_honk_evaluations(
        instance->prover_polynomials, alpha, instance->relation_parameters);

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

TEST_F(ProtoGalaxyTests, PowPerturbatorPolynomial)
{
    const size_t log_instance_size(3);
    const size_t instance_size(1 << log_instance_size);

    std::array<barretenberg::Polynomial<FF>, NUM_POLYNOMIALS> random_polynomials;
    for (auto& poly : random_polynomials) {
        poly = get_random_polynomial(instance_size);
    }
    auto full_polynomials = construct_ultra_full_polynomials(random_polynomials);
    auto relation_parameters = proof_system::RelationParameters<FF>::get_random();
    auto alpha = FF::random_element();

    auto full_honk_evals =
        ProtoGalaxyProver::compute_full_honk_evaluations(full_polynomials, alpha, relation_parameters);
    std::vector<FF> betas(log_instance_size);
    for (size_t idx = 0; idx < log_instance_size; idx++) {
        betas[idx] = FF::random_element();
    }

    // Construct pow(\vec{betas}) manually as in the paper
    std::vector<FF> pow_beta(instance_size);
    for (size_t i = 0; i < instance_size; i++) {
        auto res = FF(1);
        for (size_t j = i, beta_idx = 0; j > 0; j >>= 1, beta_idx++) {
            if ((j & 1) == 1) {
                res *= betas[beta_idx];
            }
        }
        pow_beta[i] = res;
    }

    // Compute the corresponding target sum and create a dummy accumulator
    auto target_sum = FF(0);
    for (size_t i = 0; i < instance_size; i++) {
        target_sum += full_honk_evals[i] * pow_beta[i];
    }

    auto accumulator = std::make_shared<Instance>(
        FoldingResult<Flavor>{ .folded_prover_polynomials = full_polynomials,
                               .folded_public_inputs = std::vector<FF>{},
                               .verification_key = std::make_shared<Flavor::VerificationKey>(),
                               .folding_parameters = { betas, target_sum } });
    accumulator->relation_parameters = relation_parameters;

    auto deltas = ProtoGalaxyProver::compute_round_challenge_pows(log_instance_size, FF::random_element());
    auto perturbator = ProtoGalaxyProver::compute_perturbator(accumulator, deltas, alpha);

    // Ensure the constant coefficient of the perturbator is equal to the target sum as indicated by the paper
    EXPECT_EQ(perturbator[0], target_sum);
}

TEST_F(ProtoGalaxyTests, FoldChallenges)
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
    ProtoGalaxyProver::fold_parameters(instances);

    Univariate<FF, 12> expected_eta{ { 1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23 } };
    EXPECT_EQ(instances.relation_parameters.eta, expected_eta);
}

// namespace proof_system::honk::instance_tests {

// template <class Flavor> class InstancesTests : public testing::Test {
//     using FF = typename Flavor::FF;
//     using Builder = typename Flavor::CircuitBuilder;

//   public:
//     static void test_parameters_to_univariates()
//     {

//     };
// };

// using FlavorTypes = testing::Types<flavor::Ultra>;
// TYPED_TEST_SUITE(InstancesTests, FlavorTypes);

// TYPED_TEST(InstancesTests, ParametersToUnivariates)
// {
//     TestFixture::test_parameters_to_univariates();
// }

// } // namespace proof_system::honk::instance_tests
} // namespace protogalaxy_tests