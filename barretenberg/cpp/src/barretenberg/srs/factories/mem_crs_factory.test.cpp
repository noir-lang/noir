#include "mem_crs_factory.hpp"
#include "../io.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/bn254/pairing.hpp"
#include "file_crs_factory.hpp"
#include <fstream>
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace barretenberg::srs::factories;

TEST(reference_string, mem_file_consistency)
{
    // Load 1024 from file.
    auto file_crs = FileCrsFactory<curve::BN254>("../srs_db/ignition", 1024);

    // Use low level io lib to read 1024 from file.
    std::vector<g1::affine_element> points(1024);
    ::srs::IO<curve::BN254>::read_transcript_g1(points.data(), 1024, "../srs_db/ignition");

    g2::affine_element g2_point;
    ::srs::IO<curve::BN254>::read_transcript_g2(g2_point, "../srs_db/ignition");

    MemCrsFactory mem_crs(points, g2_point);
    auto file_prover_crs = file_crs.get_prover_crs(1024);
    auto mem_prover_crs = mem_crs.get_prover_crs(1024);
    file_prover_crs->get_monomial_size();

    EXPECT_EQ(mem_prover_crs->get_monomial_size(), file_prover_crs->get_monomial_size());

    EXPECT_EQ(memcmp(mem_prover_crs->get_monomial_points(),
                     file_prover_crs->get_monomial_points(),
                     sizeof(g1::affine_element) * 1024 * 2),
              0);

    auto file_verifier_crs = file_crs.get_verifier_crs();
    auto mem_verifier_crs = file_crs.get_verifier_crs();

    EXPECT_EQ(mem_verifier_crs->get_g2x(), file_verifier_crs->get_g2x());

    EXPECT_EQ(memcmp(mem_verifier_crs->get_precomputed_g2_lines(),
                     file_verifier_crs->get_precomputed_g2_lines(),
                     sizeof(barretenberg::pairing::miller_lines) * 2),
              0);
}

TEST(reference_string, grumpkin)
{
    auto file_crs = FileCrsFactory<curve::Grumpkin>("../srs_db/grumpkin");
}
