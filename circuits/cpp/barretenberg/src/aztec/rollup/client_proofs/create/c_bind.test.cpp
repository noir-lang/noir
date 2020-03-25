#include "c_bind.h"
#include "create.hpp"
#include "../../tx/user_context.hpp"
#include <plonk/reference_string/file_reference_string.hpp>
#include <gtest/gtest.h>
#include <crypto/schnorr/schnorr.hpp>
#include <common/streams.hpp>
#include <fstream>

using namespace barretenberg;
using namespace rollup::client_proofs::create;
using namespace rollup::tx;

TEST(client_proofs, test_create_c_bindings)
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

    init_keys(monomials.data(), static_cast<uint32_t>(monomials.size()), g2x.data());

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
    create_note_proof(
        owner_buf.data(), note.value, viewing_key_buf.data(), signature.s.data(), signature.e.data(), result.data());

    uint32_t proof_length = *(uint32_t*)result.data();
    std::vector<uint8_t> proof_data(&result[4], &result[4] + proof_length);

    bool verified = verify_proof(proof_data.data(), proof_length);

    EXPECT_TRUE(verified);
}