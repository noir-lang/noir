#include "inner_proof_data.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace rollup::client_proofs;

TEST(client_proofs_inner_proof_data, test_proof_to_data)
{
    uint256_t proof_id = 0;
    uint256_t public_input = 100;
    uint256_t public_output = 20;
    std::array<uint8_t, 64> note1 = { 0x01 };
    std::array<uint8_t, 64> note2 = { 0x02 };
    auto merkle_root = fr::random_element();
    uint128_t nullifier1 = static_cast<uint128_t>(0x8e918141efe8189b) << 64 | 0x9304085fa8822c2b;
    uint128_t nullifier2 = static_cast<uint128_t>(0x4cc6a449b48527bc) << 64 | 0x3d02fd1e11a213bb;
    auto input_owner = fr::random_element();
    auto output_owner = fr::random_element();

    using serialize::write;
    std::vector<uint8_t> proof_data;
    write(proof_data, proof_id);
    write(proof_data, public_input);
    write(proof_data, public_output);
    write(proof_data, note1);
    write(proof_data, note2);
    write(proof_data, uint256_t::from_uint128(nullifier1));
    write(proof_data, uint256_t::from_uint128(nullifier2));
    write(proof_data, input_owner);
    write(proof_data, output_owner);
    write(proof_data, merkle_root);
    write(proof_data, uint256_t::from_uint128(nullifier1));

    auto data = inner_proof_data(proof_data);

    EXPECT_EQ(data.proof_id, proof_id);
    EXPECT_EQ(data.public_input, public_input);
    EXPECT_EQ(data.public_output, public_output);
    EXPECT_EQ(data.nullifier1, nullifier1);
    EXPECT_EQ(data.nullifier2, nullifier2);
    EXPECT_EQ(data.input_owner, input_owner);
    EXPECT_EQ(data.output_owner, output_owner);
    EXPECT_EQ(data.merkle_root, merkle_root);
    EXPECT_EQ(data.account_nullifier, nullifier1);
}
