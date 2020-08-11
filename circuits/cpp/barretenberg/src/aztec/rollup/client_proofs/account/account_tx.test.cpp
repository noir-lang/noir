#include "../../fixtures/user_context.hpp"
#include "c_bind.h"
#include "account.hpp"
#include <common/streams.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <ecc/curves/bn254/scalar_multiplication/c_bind.hpp>
#include <fstream>
#include <gtest/gtest.h>
#include <plonk/reference_string/pippenger_reference_string.hpp>
#include <srs/io.hpp>

using namespace barretenberg;
using namespace rollup::client_proofs::account;

TEST(client_proofs_account_tx, test_serialization)
{
    account_tx tx;
    tx.merkle_root = fr::random_element();
    tx.owner_pub_key = grumpkin::g1::element::random_element();
    tx.num_new_keys = 2;
    tx.new_signing_pub_key_1 = grumpkin::g1::element::random_element();
    tx.new_signing_pub_key_2 = grumpkin::g1::element::random_element();
    tx.register_alias = true;
    tx.alias = fr::random_element();
    tx.nullify_key = false;
    tx.nullified_key = grumpkin::g1::element::random_element();
    tx.account_index = 123;
    tx.signing_pub_key = grumpkin::g1::one * grumpkin::fr::random_element();

    for (size_t i = 0; i < 32; ++i) {
        tx.account_path.push_back(std::make_pair(fr::random_element(), fr::random_element()));
    }

    memset(&tx.signature.e, 1, 32);
    memset(&tx.signature.s, 2, 32);

    auto buffer = to_buffer(tx);
    auto tx2 = from_buffer<account_tx>(buffer.data());

    EXPECT_EQ(tx, tx2);
}