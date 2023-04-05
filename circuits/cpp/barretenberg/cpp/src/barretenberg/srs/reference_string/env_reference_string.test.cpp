#include "file_reference_string.hpp"
#include "env_reference_string.hpp"

#include "barretenberg/ecc/curves/bn254/pairing.hpp"

#include <gtest/gtest.h>

#include <fstream>

TEST(reference_string, env_file_consistency)
{
    auto env_crs = std::make_unique<proof_system::EnvReferenceStringFactory>();

    auto file_crs = std::make_unique<proof_system::FileReferenceStringFactory>("../srs_db/ignition");
    auto file_verifier = file_crs->get_verifier_crs();

    EXPECT_EQ(env_crs->get_verifier_crs()->get_g2x(), file_verifier->get_g2x());
    EXPECT_EQ(memcmp(env_crs->get_verifier_crs()->get_precomputed_g2_lines(),
                     file_verifier->get_precomputed_g2_lines(),
                     sizeof(barretenberg::pairing::miller_lines) * 2),
              0);
    EXPECT_EQ(env_crs->get_prover_crs(1)->get_monomial_points()[0],
              file_crs->get_prover_crs(1)->get_monomial_points()[0]);
}
