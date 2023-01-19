#include "sumcheck.hpp"
#include "transcript/transcript_wrappers.hpp"
#include "polynomials/multivariates.hpp"
#include "relations/arithmetic_relation.hpp"
#include "relations/grand_product_computation_relation.hpp"
#include "relations/grand_product_initialization_relation.hpp"
#include "transcript/manifest.hpp"
#include <array>
#include <cstddef>
#include <ecc/curves/bn254/fr.hpp>
#include <numeric/random/engine.hpp>

#include <initializer_list>
#include <gtest/gtest.h>

using namespace honk;
using namespace honk::sumcheck;

namespace test_sumcheck_round {

using Transcript = transcript::StandardTranscript;
using FF = barretenberg::fr;

TEST(Sumcheck, Prover)
{
    const size_t num_polys(proving_system::StandardArithmetization::NUM_POLYNOMIALS);
    const size_t multivariate_d(1);
    // const size_t multivariate_n(1 << multivariate_d);

    // const size_t max_relation_length = 4;

    using Multivariates = ::Multivariates<FF, num_polys>;

    std::array<FF, 2> w_l = { 1, 2 };
    std::array<FF, 2> w_r = { 1, 2 };
    std::array<FF, 2> w_o = { 1, 2 };
    std::array<FF, 2> z_perm = { 1, 2 };
    std::array<FF, 2> z_perm_shift = { 0, 1 };
    std::array<FF, 2> q_m = { 1, 2 };
    std::array<FF, 2> q_l = { 1, 2 };
    std::array<FF, 2> q_r = { 1, 2 };
    std::array<FF, 2> q_o = { 1, 2 };
    std::array<FF, 2> q_c = { 1, 2 };
    std::array<FF, 2> sigma_1 = { 1, 2 };
    std::array<FF, 2> sigma_2 = { 1, 2 };
    std::array<FF, 2> sigma_3 = { 1, 2 };
    std::array<FF, 2> id_1 = { 1, 2 };
    std::array<FF, 2> id_2 = { 1, 2 };
    std::array<FF, 2> id_3 = { 1, 2 };
    std::array<FF, 2> lagrange_1 = { 1, 2 };

    // These will be owned outside the class, probably by the composer.
    std::array<std::span<FF>, Multivariates::num> full_polynomials = {
        w_l, w_r,     w_o,     z_perm,  z_perm_shift, q_m,  q_l,  q_r,       q_o,
        q_c, sigma_1, sigma_2, sigma_3, id_1,         id_2, id_3, lagrange_1
    };

    // Mock prover-transcript interactions prior to Sumcheck
    auto transcript = Transcript(StandardHonk::create_unrolled_manifest(0, multivariate_d));
    transcript.mock_inputs_prior_to_challenge("alpha");
    transcript.apply_fiat_shamir("alpha");

    auto multivariates = Multivariates(full_polynomials);

    auto sumcheck = Sumcheck<Multivariates,
                             Transcript,
                             ArithmeticRelation,
                             GrandProductComputationRelation,
                             GrandProductInitializationRelation>(multivariates, transcript);

    sumcheck.execute_prover();
    // TODO(Cody) This does not constitute a test.
}

TEST(Sumcheck, Verifier)
{
    const size_t num_polys(proving_system::StandardArithmetization::NUM_POLYNOMIALS);
    const size_t multivariate_d(1);
    const size_t multivariate_n(1 << multivariate_d);
    // const size_t max_relation_length = 5;

    using Multivariates = ::Multivariates<FF, num_polys>;

    // Mock prover-transcript interactions up to and including Sumcheck
    auto transcript = Transcript(StandardHonk::create_unrolled_manifest(0, multivariate_d));
    transcript.mock_inputs_prior_to_challenge("rho", multivariate_n);

    auto sumcheck = Sumcheck<Multivariates,
                             Transcript,
                             ArithmeticRelation,
                             GrandProductComputationRelation,
                             GrandProductInitializationRelation>(transcript);

    sumcheck.execute_verifier();
    // TODO(Cody) This does not constitute a test.
}

} // namespace test_sumcheck_round
