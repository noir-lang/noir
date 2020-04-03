#include "../../pedersen_note/pedersen_note.hpp"
#include "../../tx/user_context.hpp"
#include "standard_example.hpp"
#include <common/streams.hpp>
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace plonk::stdlib::types::standard;
using namespace rollup::client_proofs::standard_example;
using namespace rollup::tx;

TEST(client_proofs, test_standard_example)
{
    Composer composer = Composer("../srs_db");

    build_circuit(composer);

    Prover prover = composer.create_prover();
    waffle::plonk_proof proof = prover.construct_proof();

    std::cout << "gates: " << composer.get_num_gates() << std::endl;
    std::cout << "proof size: " << proof.proof_data.size() << std::endl;
    std::cout << "public inputs size: " << composer.public_inputs.size() << std::endl;

    auto verifier = composer.create_verifier();
    bool result = verifier.verify_proof(proof);

    EXPECT_TRUE(result);
}