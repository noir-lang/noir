#include <barretenberg/curves/grumpkin/grumpkin.hpp>
#include <barretenberg/misc_crypto/schnorr/schnorr.hpp>
#include <barretenberg/io/streams.hpp>

#include <gtest/gtest.h>

using namespace barretenberg;

TEST(schnorr, verify_signature_keccak256)
{
    std::string message = "The quick brown fox jumped over the lazy dog.";

    crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> account;
    account.private_key = grumpkin::fr::random_element();
    account.public_key = grumpkin::g1::one * account.private_key;

    crypto::schnorr::signature signature =
        crypto::schnorr::construct_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
            message, account);

    bool result =
        crypto::schnorr::verify_signature<KeccakHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
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
        crypto::schnorr::construct_signature<Sha256Hasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
            message, account);

    bool result =
        crypto::schnorr::verify_signature<Sha256Hasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
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

    crypto::schnorr::signature signature = crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message, account);

    bool result = crypto::schnorr::verify_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message, account.public_key, signature);

    EXPECT_EQ(result, true);
}

TEST(schnorr, verify_ecrecover)
{
    std::string message = "The quick brown dog jumped over the lazy fox.";

    crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> account;
    account.private_key = grumpkin::fr::random_element();
    account.public_key = grumpkin::g1::one * account.private_key;

    crypto::schnorr::signature_b signature = crypto::schnorr::
        construct_signature_b<Sha256Hasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message,
                                                                                                        account);

    grumpkin::g1::affine_element recovered_key =
        crypto::schnorr::ecrecover<Sha256Hasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message,
                                                                                                             signature);

    EXPECT_EQ(recovered_key, account.public_key);
}