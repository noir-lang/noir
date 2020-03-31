#include "../../tx/user_context.hpp"
#include "c_bind.h"
#include "create.hpp"
#include <common/streams.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <fstream>
#include <gtest/gtest.h>
#include <plonk/reference_string/file_reference_string.hpp>
#include <srs/io.hpp>

using namespace barretenberg;
using namespace rollup::client_proofs::create;
using namespace rollup::tx;

g1::affine_element* create_pippenger_point_table(uint8_t* points, size_t num_points)
{
    g1::affine_element* monomials = (barretenberg::g1::affine_element*)(aligned_alloc(
        64, sizeof(barretenberg::g1::affine_element) * (2 * num_points + 2)));

    monomials[0] = barretenberg::g1::affine_one;

    barretenberg::io::read_g1_elements_from_buffer(&monomials[1], (char*)points, num_points * 64);
    barretenberg::scalar_multiplication::generate_pippenger_point_table(monomials, monomials, num_points);

    return monomials;
}

TEST(client_proofs, test_create_c_bindings)
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
    init_keys((uint8_t*)point_table, num_points, g2x.data());

    auto user = create_user_context();

    tx_note note = {
        user.public_key,
        100,
        user.note_secret,
    };

    auto encrypted_note = encrypt_note(note);

    std::vector<uint8_t> message(sizeof(fr));
    fr::serialize_to_buffer(encrypted_note.x, &message[0]);
    crypto::schnorr::signature signature =
        crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
            std::string(message.begin(), message.end()), { user.private_key, user.public_key });

    auto owner_buf = note.owner.to_buffer();
    auto viewing_key_buf = note.secret.to_buffer();
    std::vector<uint8_t> result(1024 * 10);
    Prover* prover = (Prover*)new_create_note_prover(
        owner_buf.data(), note.value, viewing_key_buf.data(), signature.s.data(), signature.e.data());

    auto& proof = prover->construct_proof();

    bool verified = verify_proof(proof.proof_data.data(), (uint32_t)proof.proof_data.size());

    delete_create_note_prover(prover);
    aligned_free(point_table);

    EXPECT_TRUE(verified);
}