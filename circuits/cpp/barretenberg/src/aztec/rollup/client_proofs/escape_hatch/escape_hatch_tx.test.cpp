#include "../../fixtures/user_context.hpp"
#include "c_bind.h"
#include "escape_hatch.hpp"
#include <common/streams.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <ecc/curves/bn254/scalar_multiplication/c_bind.hpp>
#include <fstream>
#include <gtest/gtest.h>
#include <plonk/reference_string/pippenger_reference_string.hpp>
#include <srs/io.hpp>

using namespace barretenberg;
using namespace rollup::client_proofs::escape_hatch;

TEST(client_proofs_escape_hatch_tx, test_serialization)
{
    escape_hatch_tx tx;
    tx.account_index = 0;
    tx.public_output = 20;
    tx.num_input_notes = 2;
    tx.input_index = { 0, 1 };
    tx.old_data_root = fr::random_element();

    for (size_t i = 0; i < 32; ++i) {
        tx.account_path.push_back(std::make_pair(fr::random_element(), fr::random_element()));
        tx.input_path[0].push_back(std::make_pair(fr::random_element(), fr::random_element()));
        tx.input_path[1].push_back(std::make_pair(fr::random_element(), fr::random_element()));
    }

    for (size_t i = 0; i < 2; ++i) {
        tx.input_note[i].value = 123;
        tx.input_note[i].secret = fr::random_element();
    }

    memset(&tx.signature.e, 1, 32);
    memset(&tx.signature.s, 2, 32);

    tx.public_owner = fr::random_element();
    tx.old_nullifier_merkle_root = fr::random_element();

    for (size_t i = 0; i < 2; ++i) {
        tx.new_null_roots[i] = fr::random_element();
    }

    for (size_t i = 0; i < 128; ++i) {
        tx.current_nullifier_paths[0].push_back(std::make_pair(fr::random_element(), fr::random_element()));
        tx.current_nullifier_paths[1].push_back(std::make_pair(fr::random_element(), fr::random_element()));
    }

    for (size_t i = 0; i < 128; ++i) {
        tx.new_nullifier_paths[0].push_back(std::make_pair(fr::random_element(), fr::random_element()));
        tx.new_nullifier_paths[1].push_back(std::make_pair(fr::random_element(), fr::random_element()));
    }

    tx.account_nullifier_path.push_back(std::make_pair(fr::random_element(), fr::random_element()));
    tx.signing_pub_key = grumpkin::g1::one * grumpkin::fr::random_element();

    tx.new_data_root = fr::random_element();
    tx.old_data_roots_root = fr::random_element();
    tx.new_data_roots_root = fr::random_element();

    auto buffer = to_buffer(tx);
    auto tx2 = from_buffer<escape_hatch_tx>(buffer.data());

    EXPECT_EQ(tx, tx2);
}
