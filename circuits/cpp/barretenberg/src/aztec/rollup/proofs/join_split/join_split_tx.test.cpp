#include "../../fixtures/user_context.hpp"

#include "c_bind.h"
#include "join_split.hpp"

#include <common/streams.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <numeric/random/engine.hpp>
#include <plonk/reference_string/pippenger_reference_string.hpp>
#include <srs/io.hpp>

#include <fstream>
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace rollup::proofs::join_split;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

TEST(client_proofs_join_split_tx, test_serialization)
{
    join_split_tx tx;
    tx.proof_id = 1;
    tx.account_note_index = 0;
    tx.signing_pub_key = grumpkin::g1::one * grumpkin::fr::random_element(&engine);
    tx.public_value = 10;
    tx.public_owner = fr::random_element(&engine);
    tx.num_input_notes = 2;
    tx.account_private_key = grumpkin::fr::random_element(&engine);
    tx.alias_hash = 0;
    tx.account_required = true;
    tx.old_data_root = fr::random_element(&engine);
    for (size_t i = 0; i < 32; ++i) {
        tx.account_note_path.push_back(std::make_pair(fr::random_element(&engine), fr::random_element(&engine)));
        tx.input_path[0].push_back(std::make_pair(fr::random_element(&engine), fr::random_element(&engine)));
        tx.input_path[1].push_back(std::make_pair(fr::random_element(&engine), fr::random_element(&engine)));
    }

    for (size_t i = 0; i < 2; ++i) {
        tx.input_note[i].owner = tx.signing_pub_key;
        tx.input_note[i].value = 123;
        tx.input_note[i].secret = fr::random_element(&engine);
        tx.input_note[i].input_nullifier = fr::random_element(&engine);
        tx.input_note[i].creator_pubkey = fr::random_element(&engine);
        tx.input_note[i].account_required = true;
    }

    for (size_t i = 0; i < 2; ++i) {
        tx.output_note[i].owner = tx.signing_pub_key;
        tx.output_note[i].value = 321;
        tx.output_note[i].secret = fr::random_element(&engine);
        tx.output_note[i].input_nullifier = fr::random_element(&engine);
        tx.output_note[i].creator_pubkey = fr::random_element(&engine);
        tx.output_note[i].account_required = true;
    }

    tx.partial_claim_note.bridge_call_data = 0xdeadbeef;
    tx.partial_claim_note.note_secret = 0xcafebabe;
    tx.partial_claim_note.deposit_value = 666;

    memset(&tx.signature.e, 1, 32);
    memset(&tx.signature.s, 2, 32);

    tx.backward_link = fr::random_element(&engine);
    tx.allow_chain = 2;

    auto buffer = to_buffer(tx);
    auto tx2 = from_buffer<join_split_tx>(buffer.data());

    EXPECT_EQ(tx, tx2);
}
