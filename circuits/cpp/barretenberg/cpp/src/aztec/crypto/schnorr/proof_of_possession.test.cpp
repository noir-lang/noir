
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <gtest/gtest.h>
#include "proof_of_possession.hpp"

using namespace barretenberg;

template <typename Hash> struct ProofOfPossessionTest : public ::testing::Test {
    using G = grumpkin::g1;
    using Fr = grumpkin::fr;
    using KeyPair = crypto::schnorr::key_pair<Fr, G>;

    static KeyPair generate_account()
    {
        KeyPair account;
        account.private_key = Fr::random_element();
        account.public_key = G::one * account.private_key;
        return account;
    }
};

using HashTypes = ::testing::Types<KeccakHasher, Sha256Hasher, Blake2sHasher>;
TYPED_TEST_SUITE(ProofOfPossessionTest, HashTypes);

TYPED_TEST(ProofOfPossessionTest, valid_proof)
{
    using G = grumpkin::g1;
    using Hash = TypeParam;
    using Proof = crypto::schnorr::ProofOfPossession<G, Hash>;

    const auto account = this->generate_account();
    const auto proof = Proof(account);
    EXPECT_TRUE(proof.verify(account.public_key));
}

TYPED_TEST(ProofOfPossessionTest, invalid_empty_proof)
{
    using G = grumpkin::g1;
    using Hash = TypeParam;
    using Proof = crypto::schnorr::ProofOfPossession<G, Hash>;

    const auto account = this->generate_account();
    const auto proof = Proof();
    EXPECT_FALSE(proof.verify(account.public_key));
}

TYPED_TEST(ProofOfPossessionTest, fail_with_different_account)
{
    using G = grumpkin::g1;
    using Hash = TypeParam;
    using Proof = crypto::schnorr::ProofOfPossession<G, Hash>;

    const auto account1 = this->generate_account();
    const auto account2 = this->generate_account();
    auto proof = Proof(account1);
    EXPECT_FALSE(proof.verify(account2.public_key));
}

TYPED_TEST(ProofOfPossessionTest, fail_zero_challenge)
{
    using G = grumpkin::g1;
    using Hash = TypeParam;
    using Proof = crypto::schnorr::ProofOfPossession<G, Hash>;

    const auto account = this->generate_account();
    auto proof = Proof(account);
    proof.challenge = {
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    };
    EXPECT_FALSE(proof.verify(account.public_key));
}

TYPED_TEST(ProofOfPossessionTest, fail_zero_response)
{
    using G = grumpkin::g1;
    using Hash = TypeParam;
    using Proof = crypto::schnorr::ProofOfPossession<G, Hash>;

    const auto account = this->generate_account();
    auto proof = Proof(account);
    // Setting the response part of the proof of posession should cause verification to fail.
    proof.response = 0;
    EXPECT_FALSE(proof.verify(account.public_key));
}

TYPED_TEST(ProofOfPossessionTest, serialize)
{
    using G = grumpkin::g1;
    using Hash = TypeParam;
    using Proof = crypto::schnorr::ProofOfPossession<G, Hash>;
    const auto account = this->generate_account();
    const auto proof = Proof(account);
    EXPECT_TRUE(proof.verify(account.public_key));

    auto buf = to_buffer(proof);
    EXPECT_EQ(buf.size(), 64);
    Proof proof2{ from_buffer<Proof, std::vector<uint8_t>>(buf) };
    EXPECT_EQ(proof.response, proof2.response);
    EXPECT_EQ(proof.challenge, proof2.challenge);

    EXPECT_TRUE(proof2.verify(account.public_key));
}