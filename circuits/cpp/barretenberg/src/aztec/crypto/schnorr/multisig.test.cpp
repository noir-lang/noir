#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <gtest/gtest.h>

#include "./multisig.hpp"

using namespace barretenberg;

using key_pair = crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1>;

using multisig = crypto::schnorr::multisig<grumpkin::g1, Blake2sHasher>;

inline key_pair generate_account()
{
    key_pair account;
    account.private_key = grumpkin::fr::random_element();
    account.public_key = grumpkin::g1::one * account.private_key;
    return account;
}

std::vector<multisig::MultiSigPublicKey> create_signer_pubkeys(const std::vector<key_pair>& accounts)
{
    // setup multisig signers
    std::vector<multisig::MultiSigPublicKey> signer_pubkeys;
    for (size_t i = 0; i < accounts.size(); ++i) {
        auto& signer = accounts[i];
        signer_pubkeys.push_back(multisig::create_multi_sig_public_key(signer));
    }
    return signer_pubkeys;
}

crypto::schnorr::signature create_multisig(const std::string& message,
                                           const std::vector<key_pair>& accounts,
                                           const bool tamper_proof_of_possession = false)
{
    std::vector<multisig::RoundOnePublicOutput> round1_pub;
    std::vector<multisig::RoundOnePrivateOutput> round1_priv;
    std::vector<multisig::RoundTwoPublicOutput> round2;
    std::vector<multisig::MultiSigPublicKey> signer_pubkeys = create_signer_pubkeys(accounts);

    if (tamper_proof_of_possession) {
        signer_pubkeys[0].proof_of_possession =
            crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>("test",
                                                                                                          accounts[0]);
    }
    const size_t num_signers = accounts.size();

    for (size_t i = 0; i < num_signers; ++i) {
        auto [_round1_pub, _round1_priv] = multisig::construct_signature_round_1();
        round1_pub.push_back(_round1_pub);
        round1_priv.push_back(_round1_priv);
    }

    for (size_t i = 0; i < num_signers; ++i) {
        auto& signer = accounts[i];
        round2.push_back(
            multisig::construct_signature_round_2(message, signer, round1_priv[i], signer_pubkeys, round1_pub));
    }

    crypto::schnorr::signature signature = multisig::combine_signatures(message, signer_pubkeys, round1_pub, round2);
    return signature;
}

TEST(multisig, verify_multi_signature_blake2s)
{
    std::string message = "The quick brown dog jumped over the lazy fox.";

    const size_t num_signers = 5;

    std::vector<key_pair> accounts(num_signers);
    for (auto& acct : accounts) {
        acct = generate_account();
    }

    auto signature = create_multisig(message, accounts);

    bool result = crypto::schnorr::verify_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message, multisig::validate_and_combine_signer_pubkeys(create_signer_pubkeys(accounts)), signature);

    EXPECT_EQ(result, true);
}

TEST(multisig, multi_signature_fails_if_proof_of_possession_invalid)
{
    std::string message = "The quick brown dog jumped over the lazy fox.";

    const size_t num_signers = 5;

    std::vector<key_pair> accounts(num_signers);
    for (auto& acct : accounts) {
        acct = generate_account();
    }

    auto signature = create_multisig(message, accounts, true);

    bool result = crypto::schnorr::verify_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message, multisig::validate_and_combine_signer_pubkeys(create_signer_pubkeys(accounts)), signature);

    EXPECT_EQ(result, false);
}

TEST(multisig, multi_signature_fails_if_duplicates)
{
    std::string message = "The quick brown dog jumped over the lazy fox.";

    const size_t num_signers = 5;

    std::vector<key_pair> accounts(num_signers);
    for (auto& acct : accounts) {
        acct = generate_account();
    }

    accounts[2] = accounts[4]; // :o
    auto signature = create_multisig(message, accounts);

    bool result = crypto::schnorr::verify_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message, multisig::validate_and_combine_signer_pubkeys(create_signer_pubkeys(accounts)), signature);

    EXPECT_EQ(result, false);
}
