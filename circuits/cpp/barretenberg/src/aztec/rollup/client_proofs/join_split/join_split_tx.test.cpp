#include "../../tx/user_context.hpp"
#include "c_bind.h"
#include <ecc/curves/bn254/scalar_multiplication/c_bind.hpp>
#include "join_split.hpp"
#include <common/streams.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <fstream>
#include <gtest/gtest.h>
#include <srs/io.hpp>
#include <plonk/reference_string/pippenger_reference_string.hpp>

using namespace barretenberg;
using namespace rollup::client_proofs::join_split;

TEST(client_proofs_join_split, test_serialization)
{
  join_split_tx tx;
  tx.owner_pub_key = grumpkin::g1::one * grumpkin::fr::random_element();
  tx.public_input = 10;
  tx.public_output = 20;
  tx.num_input_notes = 2;

  for (size_t i=0; i<32; ++i) {
    tx.input_note_hash_paths[0].push_back(std::make_pair(fr::random_element(), fr::random_element()));
    tx.input_note_hash_paths[1].push_back(std::make_pair(fr::random_element(), fr::random_element()));
  }

  for (size_t i=0; i<2; ++i) {
    tx.input_note[i].owner = tx.owner_pub_key;
    tx.input_note[i].value = 123;
    tx.input_note[i].secret = fr::random_element();
  }

  for (size_t i=0; i<2; ++i) {
    tx.output_note[i].owner = tx.owner_pub_key;
    tx.output_note[i].value = 321;
    tx.output_note[i].secret = fr::random_element();
  }

  memset(&tx.signature.e, 1, 32);
  memset(&tx.signature.s, 2, 32);

  auto buffer = tx.to_buffer();
  auto tx2 = join_split_tx::from_buffer(buffer);

  EXPECT_EQ(tx.owner_pub_key, tx2.owner_pub_key);
  EXPECT_EQ(tx.public_input, tx2.public_input);
  EXPECT_EQ(tx.public_output, tx2.public_output);
  EXPECT_EQ(tx.num_input_notes, tx2.num_input_notes);
  EXPECT_EQ(tx.input_note_hash_paths[0], tx2.input_note_hash_paths[0]);
  EXPECT_EQ(tx.input_note_hash_paths[1], tx2.input_note_hash_paths[1]);
  EXPECT_EQ(tx.input_note[0].owner, tx2.input_note[0].owner);
  EXPECT_EQ(tx.input_note[0].value, tx2.input_note[0].value);
  EXPECT_EQ(tx.input_note[0].secret, tx2.input_note[0].secret);
  EXPECT_EQ(tx.output_note[0].owner, tx2.output_note[0].owner);
  EXPECT_EQ(tx.output_note[0].value, tx2.output_note[0].value);
  EXPECT_EQ(tx.output_note[0].secret, tx2.output_note[0].secret);
  EXPECT_EQ(tx.signature.e, tx2.signature.e);
  EXPECT_EQ(tx.signature.s, tx2.signature.s);
}