#include "../../tx/user_context.hpp"
#include "c_bind.h"
#include <ecc/curves/bn254/scalar_multiplication/c_bind.hpp>
#include "create.hpp"
#include <common/streams.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <fstream>
#include <gtest/gtest.h>
#include <srs/io.hpp>
#include <plonk/reference_string/pippenger_reference_string.hpp>

using namespace barretenberg;
using namespace rollup::client_proofs::create_note;
using namespace rollup::tx;

TEST(client_proofs, test_create_c_bindings)
{
    create_note__init_proving_key();

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
    Prover* prover = (Prover*)create_note__new_prover(
        owner_buf.data(), note.value, viewing_key_buf.data(), signature.s.data(), signature.e.data());

    scalar_multiplication::Pippenger pippenger("../srs_db", 32768);
    prover->key->reference_string = std::make_shared<waffle::PippengerReferenceString>(&pippenger);

    auto& proof = prover->construct_proof();

    // Read g2x.
    std::vector<uint8_t> g2x(128);
    std::ifstream transcript;
    transcript.open("../srs_db/transcript00.dat", std::ifstream::binary);
    transcript.seekg(28 + 1024 * 1024 * 64);
    transcript.read((char*)g2x.data(), 128);
    transcript.close();

    create_note__init_verification_key(&pippenger, g2x.data());

    bool verified = create_note__verify_proof(proof.proof_data.data(), (uint32_t)proof.proof_data.size());

    create_note__delete_prover(prover);

    EXPECT_TRUE(verified);
}