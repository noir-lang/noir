#include "../../pedersen_note/pedersen_note.hpp"
#include "../../tx/user_context.hpp"
#include "join_split.hpp"
#include <common/streams.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/leveldb_store.hpp>
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace plonk::stdlib;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::client_proofs::join_split;

class client_proofs_join_split : public ::testing::Test {
  protected:
    virtual void SetUp() {
        composer = std::make_unique<Composer>("../srs_db");
        merkle_tree::LevelDbStore::destroy("/tmp/client_proofs_join_split_db");
        tree = std::make_unique<merkle_tree::LevelDbStore>("/tmp/client_proofs_join_split_db", 32);
    }

    std::unique_ptr<Composer> composer;
    std::unique_ptr<merkle_tree::LevelDbStore> tree;
};

TEST_F(client_proofs_join_split, test_0_input_notes) {}