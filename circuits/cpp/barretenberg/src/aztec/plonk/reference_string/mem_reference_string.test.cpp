#include "file_reference_string.hpp"
#include "mem_reference_string.hpp"
#include <ecc/curves/bn254/pairing.hpp>
#include <fstream>
#include <gtest/gtest.h>

TEST(reference_string, mem_file_consistency)
{
    std::ifstream transcript;
    transcript.open("../srs_db/transcript00.dat", std::ifstream::binary);
    std::vector<uint8_t> monomials(32768 * 64);
    std::vector<uint8_t> g2x(128);
    transcript.seekg(28);
    transcript.read((char*)monomials.data(), 32768 * 64);
    transcript.seekg(28 + 1024 * 1024 * 64);
    transcript.read((char*)g2x.data(), 128);
    transcript.close();

    auto mem_crs = std::make_unique<waffle::MemReferenceStringFactory>(monomials.data(), monomials.size(), g2x.data());
    auto mem_prover = mem_crs->get_prover_crs(23123);
    auto mem_verifier = mem_crs->get_verifier_crs();

    auto file_crs = std::make_unique<waffle::FileReferenceStringFactory>("../srs_db");
    auto file_prover = file_crs->get_prover_crs(23123);
    auto file_verifier = file_crs->get_verifier_crs();

    EXPECT_EQ(memcmp(mem_prover->get_monomials(), file_prover->get_monomials(), 23123 * 2 * 64), 0);
    EXPECT_EQ(mem_verifier->get_g2x(), file_verifier->get_g2x());
    EXPECT_EQ(memcmp(mem_verifier->get_precomputed_g2_lines(),
                     file_verifier->get_precomputed_g2_lines(),
                     sizeof(barretenberg::pairing::miller_lines) * 2),
              0);
}