#include "../../tx/user_context.hpp"
#include "c_bind.h"
#include "standard_example.hpp"
#include <common/log.hpp>
#include <common/streams.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <fstream>
#include <gtest/gtest.h>
#include <plonk/reference_string/pippenger_reference_string.hpp>
#include <srs/io.hpp>

using namespace barretenberg;
using namespace rollup::client_proofs::standard_example;
using namespace rollup::tx;

TEST(client_proofs, test_standard_example_c_bindings)
{
    standard_example__init_proving_key();

    Prover* prover = (Prover*)::standard_example__new_prover();

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

    standard_example__init_verification_key(&pippenger, g2x.data());

    bool verified = standard_example__verify_proof(proof.proof_data.data(), (uint32_t)proof.proof_data.size());

    standard_example__delete_prover(prover);

    EXPECT_TRUE(verified);
}