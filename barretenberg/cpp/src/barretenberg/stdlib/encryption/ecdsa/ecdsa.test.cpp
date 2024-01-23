#include "barretenberg/crypto/ecdsa/ecdsa.hpp"
#include "../../primitives/bigfield/bigfield.hpp"
#include "../../primitives/biggroup/biggroup.hpp"
#include "../../primitives/curves/secp256k1.hpp"
#include "../../primitives/curves/secp256r1.hpp"
#include "barretenberg/common/test.hpp"
#include "ecdsa.hpp"

using namespace bb;

using Builder = UltraCircuitBuilder;
using curve_ = stdlib::secp256k1<Builder>;
using curveR1 = stdlib::secp256r1<Builder>;

TEST(stdlib_ecdsa, verify_signature)
{
    Builder builder = Builder();

    // whaaablaghaaglerijgeriij
    std::string message_string = "Instructions unclear, ask again later.";

    crypto::ecdsa_key_pair<curve_::fr, curve_::g1> account;
    account.private_key = curve_::fr::random_element();
    account.public_key = curve_::g1::one * account.private_key;

    crypto::ecdsa_signature signature =
        crypto::ecdsa_construct_signature<Sha256Hasher, curve_::fq, curve_::fr, curve_::g1>(message_string, account);

    bool first_result = crypto::ecdsa_verify_signature<Sha256Hasher, curve_::fq, curve_::fr, curve_::g1>(
        message_string, account.public_key, signature);
    EXPECT_EQ(first_result, true);

    curve_::g1_bigfr_ct public_key = curve_::g1_bigfr_ct::from_witness(&builder, account.public_key);

    std::vector<uint8_t> rr(signature.r.begin(), signature.r.end());
    std::vector<uint8_t> ss(signature.s.begin(), signature.s.end());
    uint8_t vv = signature.v;

    stdlib::ecdsa_signature<Builder> sig{ curve_::byte_array_ct(&builder, rr),
                                          curve_::byte_array_ct(&builder, ss),
                                          stdlib::uint8<Builder>(&builder, vv) };

    curve_::byte_array_ct message(&builder, message_string);

    curve_::bool_ct signature_result =
        stdlib::ecdsa_verify_signature<Builder, curve_, curve_::fq_ct, curve_::bigfr_ct, curve_::g1_bigfr_ct>(
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

    crypto::ecdsa_key_pair<curveR1::fr, curveR1::g1> account;
    account.private_key = curveR1::fr::random_element();
    account.public_key = curveR1::g1::one * account.private_key;

    crypto::ecdsa_signature signature =
        crypto::ecdsa_construct_signature<Sha256Hasher, curveR1::fq, curveR1::fr, curveR1::g1>(message_string, account);

    bool first_result = crypto::ecdsa_verify_signature<Sha256Hasher, curveR1::fq, curveR1::fr, curveR1::g1>(
        message_string, account.public_key, signature);
    EXPECT_EQ(first_result, true);

    curveR1::g1_bigfr_ct public_key = curveR1::g1_bigfr_ct::from_witness(&builder, account.public_key);

    std::vector<uint8_t> rr(signature.r.begin(), signature.r.end());
    std::vector<uint8_t> ss(signature.s.begin(), signature.s.end());
    uint8_t vv = signature.v;

    stdlib::ecdsa_signature<Builder> sig{ curveR1::byte_array_ct(&builder, rr),
                                          curveR1::byte_array_ct(&builder, ss),
                                          stdlib::uint8<Builder>(&builder, vv) };

    curveR1::byte_array_ct message(&builder, message_string);

    curveR1::bool_ct signature_result =
        stdlib::ecdsa_verify_signature<Builder, curveR1, curveR1::fq_ct, curveR1::bigfr_ct, curveR1::g1_bigfr_ct>(
            message, public_key, sig);

    EXPECT_EQ(signature_result.get_value(), true);

    std::cerr << "num gates = " << builder.get_num_gates() << std::endl;
    benchmark_info(Builder::NAME_STRING, "ECDSA", "Signature Verification Test", "Gate Count", builder.get_num_gates());
    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_ecdsa, ecdsa_verify_signature_noassert_succeed)
{
    Builder builder = Builder();

    // whaaablaghaaglerijgeriij
    std::string message_string = "Instructions unclear, ask again later.";

    crypto::ecdsa_key_pair<curve_::fr, curve_::g1> account;
    account.private_key = curve_::fr::random_element();
    account.public_key = curve_::g1::one * account.private_key;

    crypto::ecdsa_signature signature =
        crypto::ecdsa_construct_signature<Sha256Hasher, curve_::fq, curve_::fr, curve_::g1>(message_string, account);

    bool first_result = crypto::ecdsa_verify_signature<Sha256Hasher, curve_::fq, curve_::fr, curve_::g1>(
        message_string, account.public_key, signature);
    EXPECT_EQ(first_result, true);

    curve_::g1_bigfr_ct public_key = curve_::g1_bigfr_ct::from_witness(&builder, account.public_key);

    std::vector<uint8_t> rr(signature.r.begin(), signature.r.end());
    std::vector<uint8_t> ss(signature.s.begin(), signature.s.end());
    uint8_t vv = signature.v;

    stdlib::ecdsa_signature<Builder> sig{
        curve_::byte_array_ct(&builder, rr),
        curve_::byte_array_ct(&builder, ss),
        stdlib::uint8<Builder>(&builder, vv),
    };

    curve_::byte_array_ct message(&builder, message_string);

    curve_::bool_ct signature_result =
        stdlib::ecdsa_verify_signature_noassert<Builder, curve_, curve_::fq_ct, curve_::bigfr_ct, curve_::g1_bigfr_ct>(
            message, public_key, sig);

    EXPECT_EQ(signature_result.get_value(), true);

    std::cerr << "num gates = " << builder.get_num_gates() << std::endl;
    benchmark_info(Builder::NAME_STRING, "ECDSA", "Signature Verification Test", "Gate Count", builder.get_num_gates());
    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_ecdsa, ecdsa_verify_signature_noassert_fail)
{
    Builder builder = Builder();

    // whaaablaghaaglerijgeriij
    std::string message_string = "Instructions unclear, ask again later.";

    crypto::ecdsa_key_pair<curve_::fr, curve_::g1> account;
    account.private_key = curve_::fr::random_element();
    account.public_key = curve_::g1::one * account.private_key;

    crypto::ecdsa_signature signature =
        crypto::ecdsa_construct_signature<Sha256Hasher, curve_::fq, curve_::fr, curve_::g1>(message_string, account);

    // tamper w. signature to make fail
    signature.r[0] += 1;

    bool first_result = crypto::ecdsa_verify_signature<Sha256Hasher, curve_::fq, curve_::fr, curve_::g1>(
        message_string, account.public_key, signature);
    EXPECT_EQ(first_result, false);

    curve_::g1_bigfr_ct public_key = curve_::g1_bigfr_ct::from_witness(&builder, account.public_key);

    std::vector<uint8_t> rr(signature.r.begin(), signature.r.end());
    std::vector<uint8_t> ss(signature.s.begin(), signature.s.end());

    stdlib::ecdsa_signature<Builder> sig{ curve_::byte_array_ct(&builder, rr),
                                          curve_::byte_array_ct(&builder, ss),
                                          27 };

    curve_::byte_array_ct message(&builder, message_string);

    curve_::bool_ct signature_result =
        stdlib::ecdsa_verify_signature_noassert<Builder, curve_, curve_::fq_ct, curve_::bigfr_ct, curve_::g1_bigfr_ct>(
            message, public_key, sig);

    EXPECT_EQ(signature_result.get_value(), false);

    std::cerr << "num gates = " << builder.get_num_gates() << std::endl;
    benchmark_info(Builder::NAME_STRING, "ECDSA", "Signature Verification Test", "Gate Count", builder.get_num_gates());
    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}
