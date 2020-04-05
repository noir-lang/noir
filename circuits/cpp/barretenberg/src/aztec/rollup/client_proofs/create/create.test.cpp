#include "../../pedersen_note/pedersen_note.hpp"
#include "../../tx/user_context.hpp"
#include "create.hpp"
#include <common/streams.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::client_proofs::create;
using namespace rollup::tx;

TEST(client_proofs, test_create)
{
    Composer composer = Composer("../srs_db");
    user_context user = create_user_context();

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

    create_note_circuit(composer, note, signature);

    Prover prover = composer.create_prover();
    waffle::plonk_proof proof = prover.construct_proof();

    std::cout << "gates: " << composer.get_num_gates() << std::endl;
    std::cout << "proof size: " << proof.proof_data.size() << std::endl;
    std::cout << "public inputs size: " << composer.public_inputs.size() << std::endl;

    auto verifier = composer.create_verifier();
    bool result = verifier.verify_proof(proof);

    EXPECT_TRUE(result);
}