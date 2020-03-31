#include "../../tx/user_context.hpp"
#include "c_bind.h"
#include "standard_example.hpp"
#include <common/streams.hpp>
#include <common/log.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <fstream>
#include <gtest/gtest.h>
#include <plonk/reference_string/file_reference_string.hpp>
#include <srs/io.hpp>

using namespace barretenberg;
using namespace rollup::client_proofs::standard_example;
using namespace rollup::tx;

namespace {
g1::affine_element* create_pippenger_point_table(uint8_t* points, size_t num_points)
{
    g1::affine_element* monomials = (g1::affine_element*)(aligned_alloc(
        64, sizeof(g1::affine_element) * (2 * num_points + 2)));

    monomials[0] = g1::affine_one;

    io::read_g1_elements_from_buffer(&monomials[1], (char*)points, num_points * 64);
    scalar_multiplication::generate_pippenger_point_table(monomials, monomials, num_points);

    return monomials;
}
}

TEST(client_proofs, test_standard_example_c_bindings)
{
    constexpr size_t num_points = 32768;
    std::ifstream transcript;
    transcript.open("../srs_db/transcript00.dat", std::ifstream::binary);
    std::vector<uint8_t> monomials(num_points * 64);
    std::vector<uint8_t> g2x(128);
    transcript.seekg(28);
    transcript.read((char*)monomials.data(), num_points * 64);
    transcript.seekg(28 + 1024 * 1024 * 64);
    transcript.read((char*)g2x.data(), 128);
    transcript.close();

    auto point_table = create_pippenger_point_table(monomials.data(), num_points);
    standard_example_init_keys((uint8_t*)point_table, num_points, g2x.data());

    Prover* prover = (Prover*)::standard_example_new_prover();

    auto& proof = prover->construct_proof();

    bool verified = standard_example_verify_proof(proof.proof_data.data(), (uint32_t)proof.proof_data.size());

    standard_example_delete_prover(prover);
    aligned_free(point_table);

    EXPECT_TRUE(verified);
}