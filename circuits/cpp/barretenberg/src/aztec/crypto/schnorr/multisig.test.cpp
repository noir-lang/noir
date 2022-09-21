#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <gtest/gtest.h>

#include "./multisig.hpp"

using namespace barretenberg;

template <typename Hash> struct MultisigTest : public ::testing::Test {
    using G = grumpkin::g1;
    using Fr = grumpkin::fr;
    using KeyPair = crypto::schnorr::key_pair<Fr, G>;
    using multisig = crypto::schnorr::multisig<G, Hash, Blake2sHasher>;
    using multisig_public_key = typename multisig::MultiSigPublicKey;

    static KeyPair generate_account()
    {
        KeyPair account;
        account.private_key = Fr::random_element();
        account.public_key = G::one * account.private_key;
        return account;
    }

    static std::vector<multisig_public_key> create_signer_pubkeys(const std::vector<KeyPair>& accounts)
    {
        // setup multisig signers
        std::vector<multisig_public_key> signer_pubkeys;
        for (size_t i = 0; i < accounts.size(); ++i) {
            auto& signer = accounts[i];
            signer_pubkeys.push_back(multisig_public_key(signer));
        }
        return signer_pubkeys;
    }

    static std::optional<crypto::schnorr::signature> create_multisig(const std::string& message,
                                                                     const std::vector<KeyPair>& accounts,
                                                                     const bool tamper_proof_of_possession = false)
    {
        std::vector<typename multisig::RoundOnePublicOutput> round1_pub;
        std::vector<typename multisig::RoundOnePrivateOutput> round1_priv;
        std::vector<typename multisig::RoundTwoPublicOutput> round2;
        std::vector<typename multisig::MultiSigPublicKey> signer_pubkeys = create_signer_pubkeys(accounts);

        if (tamper_proof_of_possession) {
            signer_pubkeys[0].proof_of_possession.response += 1;
        }
        const size_t num_signers = accounts.size();

        for (size_t i = 0; i < num_signers; ++i) {
            auto [_round1_pub, _round1_priv] = multisig::construct_signature_round_1();
            round1_pub.push_back(_round1_pub);
            round1_priv.push_back(_round1_priv);
        }

        for (size_t i = 0; i < num_signers; ++i) {
            auto& signer = accounts[i];
            if (auto round2_output = multisig::construct_signature_round_2(
                    message, signer, round1_priv[i], signer_pubkeys, round1_pub)) {
                round2.push_back(*round2_output);
            }
        }
        return multisig::combine_signatures(message, signer_pubkeys, round1_pub, round2);
    }
};

using HashTypes = ::testing::Types<KeccakHasher, Sha256Hasher>;
TYPED_TEST_SUITE(MultisigTest, HashTypes);

TYPED_TEST(MultisigTest, verify_multi_signature_blake2s)
{
    using G = grumpkin::g1;
    using Fr = grumpkin::fr;
    using Fq = grumpkin::fq;
    using KeyPair = crypto::schnorr::key_pair<Fr, G>;
    using multisig = crypto::schnorr::multisig<G, TypeParam>;

    std::string message = "The quick brown dog jumped over the lazy fox.";

    const size_t num_signers = 5;

    std::vector<KeyPair> accounts(num_signers);
    for (auto& acct : accounts) {
        acct = this->generate_account();
    }

    auto signature = this->create_multisig(message, accounts);
    ASSERT_TRUE(signature.has_value());

    auto pub_key = multisig::validate_and_combine_signer_pubkeys(this->create_signer_pubkeys(accounts));
    ASSERT_TRUE(pub_key.has_value());

    bool result = crypto::schnorr::verify_signature<Blake2sHasher, Fq, Fr, G>(message, *pub_key, *signature);

    EXPECT_EQ(result, true);
}

TYPED_TEST(MultisigTest, multi_signature_fails_if_proof_of_possession_invalid)
{
    using G = grumpkin::g1;
    using Fr = grumpkin::fr;
    using KeyPair = crypto::schnorr::key_pair<Fr, G>;

    std::string message = "The quick brown dog jumped over the lazy fox.";

    const size_t num_signers = 5;

    std::vector<KeyPair> accounts(num_signers);
    for (auto& acct : accounts) {
        acct = this->generate_account();
    }

    auto signature = this->create_multisig(message, accounts, true);
    ASSERT_FALSE(signature.has_value());
}

TYPED_TEST(MultisigTest, multi_signature_fails_if_duplicates)
{
    using G = grumpkin::g1;
    using Fr = grumpkin::fr;
    using KeyPair = crypto::schnorr::key_pair<Fr, G>;

    std::string message = "The quick brown dog jumped over the lazy fox.";

    const size_t num_signers = 5;

    std::vector<KeyPair> accounts(num_signers);
    for (auto& acct : accounts) {
        acct = this->generate_account();
    }

    accounts[2] = accounts[4]; // :o
    auto signature = this->create_multisig(message, accounts);
    ASSERT_FALSE(signature.has_value());
}
