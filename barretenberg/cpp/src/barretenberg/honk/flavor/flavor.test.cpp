#include "barretenberg/honk/flavor/ultra.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include <cstddef>
#include <gtest/gtest.h>

#pragma GCC diagnostic ignored "-Wunused-local-typedefs"
#pragma GCC diagnostic ignored "-Wunused-variable"

namespace proof_system::test_flavor {
TEST(Flavor, Getters)
{
    using Flavor = proof_system::honk::flavor::Ultra;
    using FF = Flavor::FF;
    using ProvingKey = typename Flavor::ProvingKey;

    ProvingKey proving_key = []() { return Flavor::ProvingKey(/*circuit_size=*/4, /*num_public_inputs=*/0); }();

    // set
    size_t coset_idx = 0;
    for (auto& id_poly : proving_key.get_id_polynomials()) {
        typename Flavor::Polynomial new_poly(proving_key.circuit_size);
        for (size_t i = 0; i < proving_key.circuit_size; ++i) {
            id_poly[i] = coset_idx * proving_key.circuit_size + i;
        }
        ++coset_idx;
    }

    // Polynomials in the proving key can be set through loops over subsets produced by the getters
    EXPECT_EQ(proving_key.id_1[0], FF(0));
    EXPECT_EQ(proving_key.id_2[0], FF(4));
    EXPECT_EQ(proving_key.id_3[0], FF(8));

    Flavor::VerificationKey verification_key;
    Flavor::ProverPolynomials prover_polynomials;
    Flavor::ExtendedEdges<Flavor::NUM_ALL_ENTITIES> edges;
    Flavor::AllValues evals;
    Flavor::CommitmentLabels commitment_labels;

    // Globals are also available through STL container sizes
    EXPECT_EQ(prover_polynomials.size(), Flavor::NUM_ALL_ENTITIES);
    // Shited polynomials have the righ tsize
    EXPECT_EQ(prover_polynomials.size(), prover_polynomials.get_unshifted_then_shifted().size());
    // Commitment lables are stored in the flavor.
    EXPECT_EQ(commitment_labels.w_r, "W_R");

    auto get_test_polynomial = [](size_t& idx) {
        Flavor::Polynomial poly(4);
        for (size_t i = 0; i < 4; i++) {
            poly[i] = idx++;
        };
        return poly;
    };

    size_t idx = 0;
    auto w_l = get_test_polynomial(idx);
    auto w_r = get_test_polynomial(idx);
    auto w_o = get_test_polynomial(idx);
    auto w_4 = get_test_polynomial(idx);
    auto z_perm = get_test_polynomial(idx);
    auto z_perm_shift = get_test_polynomial(idx);
    auto q_m = get_test_polynomial(idx);
    auto q_l = get_test_polynomial(idx);
    auto q_r = get_test_polynomial(idx);
    auto q_o = get_test_polynomial(idx);
    auto q_c = get_test_polynomial(idx);
    auto sigma_1 = get_test_polynomial(idx);
    auto sigma_2 = get_test_polynomial(idx);
    auto sigma_3 = get_test_polynomial(idx);
    auto id_1 = get_test_polynomial(idx);
    auto id_2 = get_test_polynomial(idx);
    auto id_3 = get_test_polynomial(idx);
    auto lagrange_first = get_test_polynomial(idx);
    auto lagrange_last = get_test_polynomial(idx);

    prover_polynomials.w_l = w_l;
    prover_polynomials.w_r = w_r;
    prover_polynomials.w_o = w_o;
    prover_polynomials.w_4 = w_4;
    prover_polynomials.z_perm = z_perm;
    prover_polynomials.z_perm_shift = z_perm_shift;
    prover_polynomials.q_m = q_m;
    prover_polynomials.q_l = q_l;
    prover_polynomials.q_r = q_r;
    prover_polynomials.q_o = q_o;
    prover_polynomials.q_c = q_c;
    prover_polynomials.sigma_1 = sigma_1;
    prover_polynomials.sigma_2 = sigma_2;
    prover_polynomials.sigma_3 = sigma_3;
    prover_polynomials.id_1 = id_1;
    prover_polynomials.id_2 = id_2;
    prover_polynomials.id_3 = id_3;
    prover_polynomials.lagrange_first = lagrange_first;
    prover_polynomials.lagrange_last = lagrange_last;

    // You can set polynomial values directly through the symbol names
    // and then access the values through the getters.
    idx = 0;
    for (auto& poly : prover_polynomials.get_wires()) {
        EXPECT_EQ(poly[0], 4 * idx);
        EXPECT_EQ(poly[1], 4 * idx + 1);
        EXPECT_EQ(poly[2], 4 * idx + 2);
        EXPECT_EQ(poly[3], 4 * idx + 3);
        ++idx;
    };
}

TEST(Flavor, AllEntitiesSpecialMemberFunctions)
{
    using Flavor = proof_system::honk::flavor::Ultra;
    using FF = Flavor::FF;
    using PartiallyEvaluatedMultivariates = Flavor::PartiallyEvaluatedMultivariates;
    using Polynomial = barretenberg::Polynomial<FF>;

    PartiallyEvaluatedMultivariates polynomials_A;
    auto random_poly = Polynomial(10);
    for (auto& coeff : random_poly) {
        coeff = FF::random_element();
    }

    // Test some special member functions.

    polynomials_A.w_l = random_poly;

    ASSERT_EQ(random_poly, polynomials_A.w_l);

    PartiallyEvaluatedMultivariates polynomials_B(polynomials_A);
    ASSERT_EQ(random_poly, polynomials_B.w_l);

    PartiallyEvaluatedMultivariates polynomials_C(std::move(polynomials_B));
    ASSERT_EQ(random_poly, polynomials_C.w_l);
}

TEST(Flavor, GetRow)
{
    using Flavor = proof_system::honk::flavor::Ultra;
    using FF = typename Flavor::FF;
    std::array<std::vector<FF>, Flavor::NUM_ALL_ENTITIES> data;
    std::generate(data.begin(), data.end(), []() {
        return std::vector<FF>({ FF::random_element(), FF::random_element() });
    });
    Flavor::ProverPolynomials prover_polynomials;
    size_t poly_idx = 0;
    for (auto& poly : prover_polynomials) {
        poly = data[poly_idx];
        poly_idx++;
    }
    auto row0 = prover_polynomials.get_row(0);
    auto row1 = prover_polynomials.get_row(1);
    EXPECT_EQ(row0.q_elliptic, prover_polynomials.q_elliptic[0]);
    EXPECT_EQ(row1.w_4_shift, prover_polynomials.w_4_shift[1]);
}
} // namespace proof_system::test_flavor
