#include "schnorr.hpp"
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace crypto::schnorr;

crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> generate_signature()
{
    crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> account;
    account.private_key = grumpkin::fr::random_element();
    account.public_key = grumpkin::g1::one * account.private_key;
    return account;
}

TEST(schnorr, verify_signature_keccak256)
{
    std::string message = "The quick brown fox jumped over the lazy dog.";

    crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> account;
    account.private_key = grumpkin::fr::random_element();
    account.public_key = grumpkin::g1::one * account.private_key;

    crypto::schnorr::signature signature =
        crypto::schnorr::construct_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message, account);

    bool result = crypto::schnorr::verify_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message, account.public_key, signature);

    EXPECT_EQ(result, true);
}

TEST(schnorr, verify_signature_sha256)
{
    std::string message = "The quick brown dog jumped over the lazy fox.";

    crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> account;
    account.private_key = grumpkin::fr::random_element();
    account.public_key = grumpkin::g1::one * account.private_key;

    crypto::schnorr::signature signature =
        crypto::schnorr::construct_signature<Sha256Hasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message, account);

    bool result = crypto::schnorr::verify_signature<Sha256Hasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message, account.public_key, signature);

    EXPECT_EQ(result, true);
}

TEST(schnorr, verify_signature_blake2s)
{
    std::string message = "The quick brown dog jumped over the lazy fox.";

    crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> account;
    // account.private_key = grumpkin::fr::random_element();
    account.private_key = { 0x55555555, 0x55555555, 0x55555555, 0x55555555 };
    account.public_key = grumpkin::g1::one * account.private_key;

    crypto::schnorr::signature signature =
        crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message, account);

    bool result = crypto::schnorr::verify_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message, account.public_key, signature);

    EXPECT_EQ(result, true);
}

TEST(schnorr, hmac_signature_consistency)
{
    std::string message_a = "The quick brown fox jumped over the lazy dog.";
    std::string message_b = "The quick brown dog jumped over the lazy fox.";

    auto account_a = generate_signature();
    auto account_b = generate_signature();

    ASSERT_NE(account_a.private_key, account_b.private_key);
    ASSERT_NE(account_a.public_key, account_b.public_key);

    // k is no longer identical, so signatures should be different.
    auto signature_a =
        construct_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message_a, account_a);
    auto signature_b =
        construct_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message_a, account_a);

    ASSERT_NE(signature_a.e, signature_b.e);
    ASSERT_NE(signature_a.s, signature_b.s);

    // same message, different accounts should give different sigs!
    auto signature_c =
        construct_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message_a, account_a);
    auto signature_d =
        construct_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message_a, account_b);

    ASSERT_NE(signature_c.e, signature_d.e);
    ASSERT_NE(signature_c.s, signature_d.s);

    // different message, same accounts should give different sigs!
    auto signature_e =
        construct_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message_a, account_a);
    auto signature_f =
        construct_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message_b, account_a);

    ASSERT_NE(signature_e.e, signature_f.e);
    ASSERT_NE(signature_e.s, signature_f.s);

    // different message, different accounts should give different sigs!!
    auto signature_g =
        construct_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message_a, account_a);
    auto signature_h =
        construct_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message_b, account_b);

    ASSERT_NE(signature_g.e, signature_h.e);
    ASSERT_NE(signature_g.s, signature_h.s);

    bool res = verify_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message_a, account_a.public_key, signature_a);
    EXPECT_EQ(res, true);
    res = verify_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message_a, account_a.public_key, signature_b);
    EXPECT_EQ(res, true);
    res = verify_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message_a, account_a.public_key, signature_c);
    EXPECT_EQ(res, true);
    res = verify_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message_a, account_b.public_key, signature_d);
    EXPECT_EQ(res, true);
    res = verify_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message_a, account_a.public_key, signature_e);
    EXPECT_EQ(res, true);
    res = verify_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message_b, account_a.public_key, signature_f);
    EXPECT_EQ(res, true);
    res = verify_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message_a, account_a.public_key, signature_g);
    EXPECT_EQ(res, true);
    res = verify_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message_b, account_b.public_key, signature_h);
    EXPECT_EQ(res, true);
}