#include "barretenberg/crypto/ecdsa/ecdsa.hpp"
#include "../../primitives/bigfield/bigfield.hpp"
#include "../../primitives/biggroup/biggroup.hpp"
#include "../../primitives/curves/secp256k1.hpp"
#include "../../primitives/curves/secp256r1.hpp"
#include "barretenberg/common/test.hpp"
#include "ecdsa.hpp"

using namespace bb;
using namespace bb::plonk;

namespace test_stdlib_ecdsa {
using Builder = bb::UltraCircuitBuilder;
using curve = stdlib::secp256k1<Builder>;
using curveR1 = stdlib::secp256r1<Builder>;

TEST(stdlib_ecdsa, verify_signature)
{
    Builder builder = Builder();

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

    curve::g1_bigfr_ct public_key = curve::g1_bigfr_ct::from_witness(&builder, account.public_key);

    std::vector<uint8_t> rr(signature.r.begin(), signature.r.end());
    std::vector<uint8_t> ss(signature.s.begin(), signature.s.end());
    uint8_t vv = signature.v;

    stdlib::ecdsa::signature<Builder> sig{ curve::byte_array_ct(&builder, rr),
                                           curve::byte_array_ct(&builder, ss),
                                           stdlib::uint8<Builder>(&builder, vv) };

    curve::byte_array_ct message(&builder, message_string);

    curve::bool_ct signature_result =
        stdlib::ecdsa::verify_signature<Builder, curve, curve::fq_ct, curve::bigfr_ct, curve::g1_bigfr_ct>(
            message, public_key, sig);

    EXPECT_EQ(signature_result.get_value(), true);

    std::cerr << "num gates = " << builder.get_num_gates() << std::endl;
    benchmark_info(Builder::NAME_STRING, "ECDSA", "Signature Verification Test", "Gate Count", builder.get_num_gates());
    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_ecdsa, verify_r1_signature)
{
    Builder builder = Builder();

    std::string message_string = "Instructions unclear, ask again later.";

    crypto::ecdsa::key_pair<curveR1::fr, curveR1::g1> account;
    account.private_key = curveR1::fr::random_element();
    account.public_key = curveR1::g1::one * account.private_key;

    crypto::ecdsa::signature signature =
        crypto::ecdsa::construct_signature<Sha256Hasher, curveR1::fq, curveR1::fr, curveR1::g1>(message_string,
                                                                                                account);

    bool first_result = crypto::ecdsa::verify_signature<Sha256Hasher, curveR1::fq, curveR1::fr, curveR1::g1>(
        message_string, account.public_key, signature);
    EXPECT_EQ(first_result, true);

    curveR1::g1_bigfr_ct public_key = curveR1::g1_bigfr_ct::from_witness(&builder, account.public_key);

    std::vector<uint8_t> rr(signature.r.begin(), signature.r.end());
    std::vector<uint8_t> ss(signature.s.begin(), signature.s.end());
    uint8_t vv = signature.v;

    stdlib::ecdsa::signature<Builder> sig{ curveR1::byte_array_ct(&builder, rr),
                                           curveR1::byte_array_ct(&builder, ss),
                                           stdlib::uint8<Builder>(&builder, vv) };

    curveR1::byte_array_ct message(&builder, message_string);

    curveR1::bool_ct signature_result =
        stdlib::ecdsa::verify_signature<Builder, curveR1, curveR1::fq_ct, curveR1::bigfr_ct, curveR1::g1_bigfr_ct>(
            message, public_key, sig);

    EXPECT_EQ(signature_result.get_value(), true);

    std::cerr << "num gates = " << builder.get_num_gates() << std::endl;
    benchmark_info(Builder::NAME_STRING, "ECDSA", "Signature Verification Test", "Gate Count", builder.get_num_gates());
    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_ecdsa, verify_signature_noassert_succeed)
{
    Builder builder = Builder();

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

    curve::g1_bigfr_ct public_key = curve::g1_bigfr_ct::from_witness(&builder, account.public_key);

    std::vector<uint8_t> rr(signature.r.begin(), signature.r.end());
    std::vector<uint8_t> ss(signature.s.begin(), signature.s.end());
    uint8_t vv = signature.v;

    stdlib::ecdsa::signature<Builder> sig{
        curve::byte_array_ct(&builder, rr),
        curve::byte_array_ct(&builder, ss),
        stdlib::uint8<Builder>(&builder, vv),
    };

    curve::byte_array_ct message(&builder, message_string);

    curve::bool_ct signature_result =
        stdlib::ecdsa::verify_signature_noassert<Builder, curve, curve::fq_ct, curve::bigfr_ct, curve::g1_bigfr_ct>(
            message, public_key, sig);

    EXPECT_EQ(signature_result.get_value(), true);

    std::cerr << "num gates = " << builder.get_num_gates() << std::endl;
    benchmark_info(Builder::NAME_STRING, "ECDSA", "Signature Verification Test", "Gate Count", builder.get_num_gates());
    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_ecdsa, verify_signature_noassert_fail)
{
    Builder builder = Builder();

    // whaaablaghaaglerijgeriij
    std::string message_string = "Instructions unclear, ask again later.";

    crypto::ecdsa::key_pair<curve::fr, curve::g1> account;
    account.private_key = curve::fr::random_element();
    account.public_key = curve::g1::one * account.private_key;

    crypto::ecdsa::signature signature =
        crypto::ecdsa::construct_signature<Sha256Hasher, curve::fq, curve::fr, curve::g1>(message_string, account);

    // tamper w. signature to make fail
    signature.r[0] += 1;

    bool first_result = crypto::ecdsa::verify_signature<Sha256Hasher, curve::fq, curve::fr, curve::g1>(
        message_string, account.public_key, signature);
    EXPECT_EQ(first_result, false);

    curve::g1_bigfr_ct public_key = curve::g1_bigfr_ct::from_witness(&builder, account.public_key);

    std::vector<uint8_t> rr(signature.r.begin(), signature.r.end());
    std::vector<uint8_t> ss(signature.s.begin(), signature.s.end());

    stdlib::ecdsa::signature<Builder> sig{ curve::byte_array_ct(&builder, rr), curve::byte_array_ct(&builder, ss), 27 };

    curve::byte_array_ct message(&builder, message_string);

    curve::bool_ct signature_result =
        stdlib::ecdsa::verify_signature_noassert<Builder, curve, curve::fq_ct, curve::bigfr_ct, curve::g1_bigfr_ct>(
            message, public_key, sig);

    EXPECT_EQ(signature_result.get_value(), false);

    std::cerr << "num gates = " << builder.get_num_gates() << std::endl;
    benchmark_info(Builder::NAME_STRING, "ECDSA", "Signature Verification Test", "Gate Count", builder.get_num_gates());
    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}
} // namespace test_stdlib_ecdsa
