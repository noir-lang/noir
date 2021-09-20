#include "../../fixtures/user_context.hpp"
#include "c_bind.h"
#include "join_split.hpp"
#include <common/streams.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <ecc/curves/bn254/scalar_multiplication/c_bind.hpp>
#include <fstream>
#include <gtest/gtest.h>
#include <plonk/reference_string/pippenger_reference_string.hpp>
#include <srs/io.hpp>

using namespace barretenberg;
using namespace rollup::proofs::join_split;

TEST(client_proofs_join_split_tx, test_serialization)
{
    join_split_tx tx;
    tx.account_index = 0;
    tx.signing_pub_key = grumpkin::g1::one * grumpkin::fr::random_element();
    tx.public_input = 10;
    tx.public_output = 20;
    tx.public_owner = fr::random_element();
    tx.num_input_notes = 2;
    tx.account_private_key = grumpkin::fr::random_element();
    tx.alias_hash = 0;
    tx.nonce = 456;
    tx.old_data_root = fr::random_element();
    for (size_t i = 0; i < 32; ++i) {
        tx.account_path.push_back(std::make_pair(fr::random_element(), fr::random_element()));
        tx.input_path[0].push_back(std::make_pair(fr::random_element(), fr::random_element()));
        tx.input_path[1].push_back(std::make_pair(fr::random_element(), fr::random_element()));
    }

    for (size_t i = 0; i < 2; ++i) {
        tx.input_note[i].owner = tx.signing_pub_key;
        tx.input_note[i].value = 123;
        tx.input_note[i].secret = fr::random_element();
    }

    for (size_t i = 0; i < 2; ++i) {
        tx.output_note[i].owner = tx.signing_pub_key;
        tx.output_note[i].value = 321;
        tx.output_note[i].secret = fr::random_element();
    }

    tx.claim_note.bridge_id = 0xdeadbeef;
    tx.claim_note.note_secret = 0xcafebabe;
    tx.claim_note.deposit_value = 666;

    memset(&tx.signature.e, 1, 32);
    memset(&tx.signature.s, 2, 32);

    auto buffer = to_buffer(tx);
    auto tx2 = from_buffer<join_split_tx>(buffer.data());

    EXPECT_EQ(tx, tx2);
}
