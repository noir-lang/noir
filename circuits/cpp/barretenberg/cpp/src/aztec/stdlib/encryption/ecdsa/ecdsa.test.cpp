#include "../../primitives/bigfield/bigfield.hpp"
#include "../../primitives/biggroup/biggroup.hpp"
#include "../../primitives/curves/secp256k1.hpp"
#include "ecdsa.hpp"

#include <crypto/ecdsa/ecdsa.hpp>
#include <common/test.hpp>

using namespace barretenberg;
using namespace plonk;

namespace test_stdlib_ecdsa {
using Composer = waffle::UltraComposer;
using curve = stdlib::secp256k1<Composer>;

TEST(stdlib_ecdsa, verify_signature)
{
    Composer composer = Composer();

    // whaaablaghaaglerijgeriij
    std::string message_string = "Instructions unclear, ask again later.";

    crypto::ecdsa::key_pair<curve::fr, curve::g1> account;
    account.private_key = curve::fr::random_element();
    account.public_key = curve::g1::one * account.private_key;

    crypto::ecdsa::signature signature =
        crypto::ecdsa::construct_signature<Sha256Hasher, curve::fq, curve::fr, curve::g1>(message_string, account);

    bool first_result = crypto::ecdsa::verify_signature<Sha256Hasher, curve::fq, curve::fr, curve::g1>(
        message_string, account.public_key, signature);
    EXPECT_EQ(first_result, true);

    curve::g1_bigfr_ct public_key = curve::g1_bigfr_ct::from_witness(&composer, account.public_key);

    std::vector<uint8_t> rr(signature.r.begin(), signature.r.end());
    std::vector<uint8_t> ss(signature.s.begin(), signature.s.end());

    stdlib::ecdsa::signature<Composer> sig{ curve::byte_array_ct(&composer, rr), curve::byte_array_ct(&composer, ss) };

    curve::byte_array_ct message(&composer, message_string);

    curve::bool_ct signature_result =
        stdlib::ecdsa::verify_signature<Composer, curve, curve::fq_ct, curve::bigfr_ct, curve::g1_bigfr_ct>(
            message, public_key, sig);

    EXPECT_EQ(signature_result.get_value(), true);

    std::cerr << "composer gates = " << composer.get_num_gates() << std::endl;
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}
} // namespace test_stdlib_ecdsa