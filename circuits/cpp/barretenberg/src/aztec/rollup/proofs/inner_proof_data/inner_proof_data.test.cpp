#include "inner_proof_data.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace rollup::proofs;

namespace {
auto& rand_engine = numeric::random::get_debug_engine();
}

TEST(client_proofs_inner_proof_data, test_proof_to_data)
{
    uint256_t proof_id = 0;
    auto note1 = fr::random_element();
    auto note2 = fr::random_element();
    auto merkle_root = fr::random_element();
    uint256_t nullifier1 = rand_engine.get_random_uint256();
    uint256_t nullifier2 = rand_engine.get_random_uint256();
    uint256_t public_value = rand_engine.get_random_uint256();
    auto public_owner = fr::random_element();
    uint256_t asset_id = 1;
    uint256_t tx_fee = rand_engine.get_random_uint256();
    uint256_t tx_fee_asset_id = 2;
    uint256_t bridge_id = rand_engine.get_random_uint256();
    uint256_t defi_deposit_value = rand_engine.get_random_uint256();
    auto defi_root = fr::random_element();
    auto backward_link = fr::random_element();
    uint256_t allow_chain = 0;

    using serialize::write;
    std::vector<uint8_t> proof_data;
    write(proof_data, proof_id);
    write(proof_data, note1);
    write(proof_data, note2);
    write(proof_data, nullifier1);
    write(proof_data, nullifier2);
    write(proof_data, public_value);
    write(proof_data, public_owner);
    write(proof_data, asset_id);
    write(proof_data, merkle_root);
    write(proof_data, tx_fee);
    write(proof_data, tx_fee_asset_id);
    write(proof_data, bridge_id);
    write(proof_data, defi_deposit_value);
    write(proof_data, defi_root);
    write(proof_data, backward_link);
    write(proof_data, allow_chain);

    auto data = inner_proof_data(proof_data);

    EXPECT_EQ(data.proof_id, proof_id);
    EXPECT_EQ(data.note_commitment1, note1);
    EXPECT_EQ(data.note_commitment2, note2);
    EXPECT_EQ(data.nullifier1, nullifier1);
    EXPECT_EQ(data.nullifier2, nullifier2);
    EXPECT_EQ(data.public_value, public_value);
    EXPECT_EQ(data.public_owner, public_owner);
    EXPECT_EQ(data.asset_id, asset_id);
    EXPECT_EQ(data.merkle_root, merkle_root);
    EXPECT_EQ(data.tx_fee, tx_fee);
    EXPECT_EQ(data.tx_fee_asset_id, tx_fee_asset_id);
    EXPECT_EQ(data.bridge_id, bridge_id);
    EXPECT_EQ(data.defi_deposit_value, defi_deposit_value);
    EXPECT_EQ(data.defi_root, defi_root);
    EXPECT_EQ(data.backward_link, backward_link);
    EXPECT_EQ(data.allow_chain, allow_chain);
}
