#include "c_bind.h"
#include "function_leaf_preimage.hpp"
#include "tx_request.hpp"

#include "aztec3/circuits/abis/new_contract_data.hpp"
#include "aztec3/circuits/abis/signed_tx_request.hpp"
#include "aztec3/circuits/hash.hpp"

#include <barretenberg/common/serialize.hpp>
#include <barretenberg/numeric/random/engine.hpp>
#include <barretenberg/serialize/test_helper.hpp>
#include <barretenberg/stdlib/merkle_tree/membership.hpp>

#include <gtest/gtest.h>

namespace {

using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::NewContractData;
// num_leaves = 2**h = 2<<(h-1)
// root layer does not count in height
constexpr size_t FUNCTION_TREE_NUM_LEAVES = 2 << (aztec3::FUNCTION_TREE_HEIGHT - 1);
// num_nodes = (2**(h+1))-1 = (2<<h)
// root layer does not count in height
// num nodes includes root
constexpr size_t FUNCTION_TREE_NUM_NODES = (2 << aztec3::FUNCTION_TREE_HEIGHT) - 1;

auto& engine = numeric::random::get_debug_engine();

/**
 * @brief Convert a bytes array to a hex string.
 *
 * @details convert each byte to two hex characters
 *
 * @tparam NUM_BYTES length of bytes array input
 * @param bytes array of bytes to be converted to hex string
 * @return a string containing the hex representation of the NUM_BYTES bytes of the input array
 */
template <size_t NUM_BYTES> std::string bytes_to_hex_str(std::array<uint8_t, NUM_BYTES> bytes)
{
    std::ostringstream stream;
    for (const uint8_t& byte : bytes) {
        stream << std::setw(2) << std::setfill('0') << std::hex << static_cast<int>(byte);
    }
    return stream.str();
}

}  // namespace

namespace aztec3::circuits::abis {

TEST(abi_tests, compute_contract_address)
{
    auto func = aztec3::circuits::compute_contract_address<NT>;
    auto [actual, expected] =
        call_func_and_wrapper(func, abis__compute_contract_address, NT::address(1), NT::fr(2), NT::fr(3), NT::fr(4));
    EXPECT_EQ(actual, expected);
}
TEST(abi_tests, hash_tx_request)
{
    // randomize function args for tx request
    std::array<fr, ARGS_LENGTH> args;
    for (size_t i = 0; i < ARGS_LENGTH; i++) {
        args[i] = NT::fr::random_element();
    }

    // Construct TxRequest with some randomized fields
    TxRequest<NT> const tx_request = TxRequest<NT>{
        .from = NT::fr::random_element(),
        .to = NT::fr::random_element(),
        .function_data = FunctionData<NT>(),
        .args = args,
        .nonce = NT::fr::random_element(),
        .tx_context = TxContext<NT>(),
        .chain_id = NT::fr::random_element(),
    };

    // Write the tx request to a buffer and
    std::vector<uint8_t> buf;
    write(buf, tx_request);

    // create an output buffer for cbind hash results
    std::array<uint8_t, sizeof(NT::fr)> output = { 0 };
    // Make the c_bind call to hash the tx request
    abis__hash_tx_request(buf.data(), output.data());

    // Convert buffer to `fr` for comparison to in-test calculated hash
    NT::fr const got_hash = NT::fr::serialize_from_buffer(output.data());

    // Confirm cbind output == hash of tx request
    EXPECT_EQ(got_hash, tx_request.hash());
}

TEST(abi_tests, compute_function_selector_transfer)
{
    const char* function_signature = "transfer(address,uint256)";

    // create an output buffer for cbind selector results
    std::array<uint8_t, FUNCTION_SELECTOR_NUM_BYTES> output = { 0 };
    // Make the c_bind call to compute the function selector via keccak256
    abis__compute_function_selector(function_signature, output.data());

    // get the selector as a hex string
    // compare against known good selector from solidity
    // In solidity where selectors are 4 bytes it is a9059cbb
    std::string const full_selector = "a9059cbb2ab09eb219583f4a59a5d0623ade346d962bcd4e46b11da047c9049b";
    EXPECT_EQ(bytes_to_hex_str(output), full_selector.substr(0, FUNCTION_SELECTOR_NUM_BYTES * 2));
}

TEST(abi_tests, compute_function_selector_transferFrom)
{
    const char* function_signature = "transferFrom(address,address,uint256)";

    // create an output buffer for cbind selector results
    std::array<uint8_t, FUNCTION_SELECTOR_NUM_BYTES> output = { 0 };
    // Make the c_bind call to compute the function selector via keccak256
    abis__compute_function_selector(function_signature, output.data());

    // get the selector as a hex string
    // compare against known good selector from solidity
    std::string const full_selector = "23b872dd7302113369cda2901243429419bec145408fa8b352b3dd92b66c680b";
    EXPECT_EQ(bytes_to_hex_str(output), full_selector.substr(0, FUNCTION_SELECTOR_NUM_BYTES * 2));
}

TEST(abi_tests, hash_vk)
{
    // Initialize some random VK data
    NT::VKData vk_data;
    vk_data.composer_type = engine.get_random_uint32();
    vk_data.circuit_size = static_cast<uint32_t>(1) << (engine.get_random_uint8() >> 3);  // must be a power of two
    vk_data.num_public_inputs = engine.get_random_uint32();
    vk_data.commitments["test1"] = g1::element::random_element();
    vk_data.commitments["test2"] = g1::element::random_element();
    vk_data.commitments["foo1"] = g1::element::random_element();
    vk_data.commitments["foo2"] = g1::element::random_element();
    // Write the vk data to a bytes vector
    std::vector<uint8_t> vk_data_vec;
    write(vk_data_vec, vk_data);

    // create an output buffer for cbind hash results
    std::array<uint8_t, sizeof(NT::fr)> output = { 0 };

    // Make the c_bind call to hash the vk
    abis__hash_vk(vk_data_vec.data(), output.data());

    // Convert buffer to `fr` for comparison to in-test calculated hash
    NT::fr const got_hash = NT::fr::serialize_from_buffer(output.data());

    // Calculate the expected hash in-test
    NT::fr const expected_hash = vk_data.compress_native(aztec3::GeneratorIndex::VK);

    // Confirm cbind output == expected hash
    EXPECT_EQ(got_hash, expected_hash);
}

TEST(abi_tests, compute_function_leaf)
{
    // Construct FunctionLeafPreimage with some randomized fields
    auto const preimage = FunctionLeafPreimage<NT>{
        .function_selector = engine.get_random_uint32(),
        .is_private = static_cast<bool>(engine.get_random_uint8() & 1),
        .vk_hash = NT::fr::random_element(),
        .acir_hash = NT::fr::random_element(),
    };

    // Write the leaf preimage to a buffer
    std::vector<uint8_t> preimage_buf;
    write(preimage_buf, preimage);

    std::array<uint8_t, sizeof(NT::fr)> output = { 0 };
    abis__compute_function_leaf(preimage_buf.data(), output.data());

    NT::fr const got_leaf = NT::fr::serialize_from_buffer(output.data());
    EXPECT_EQ(got_leaf, preimage.hash());
}

TEST(abi_tests, compute_function_tree_root)
{
    // randomize number of non-zero leaves such that `0 < num_nonzero_leaves <= FUNCTION_TREE_NUM_LEAVES`
    uint8_t const num_nonzero_leaves = engine.get_random_uint8() % (FUNCTION_TREE_NUM_LEAVES + 1);

    // generate some random leaves
    std::vector<NT::fr> leaves_frs;
    for (size_t l = 0; l < num_nonzero_leaves; l++) {
        leaves_frs.push_back(NT::fr::random_element());
    }
    // serilalize the leaves to a buffer to pass to cbind
    std::vector<uint8_t> leaves_bytes_vec;
    write(leaves_bytes_vec, leaves_frs);

    // call cbind and get output (root)
    std::array<uint8_t, sizeof(NT::fr)> output = { 0 };
    abis__compute_function_tree_root(leaves_bytes_vec.data(), output.data());
    NT::fr const got_root = NT::fr::serialize_from_buffer(output.data());

    // compare cbind results with direct computation

    // add the zero leaves to the vector of fields and pass to barretenberg helper
    NT::fr const zero_leaf = FunctionLeafPreimage<NT>().hash();  // hash of empty/0 preimage
    for (size_t l = num_nonzero_leaves; l < FUNCTION_TREE_NUM_LEAVES; l++) {
        leaves_frs.push_back(zero_leaf);
    }
    // compare results
    EXPECT_EQ(got_root, plonk::stdlib::merkle_tree::compute_tree_root_native(leaves_frs));
}

TEST(abi_tests, compute_function_tree)
{
    // randomize number of non-zero leaves such that `0 < num_nonzero_leaves <= FUNCTION_TREE_NUM_LEAVES`
    uint8_t const num_nonzero_leaves = engine.get_random_uint8() % (FUNCTION_TREE_NUM_LEAVES + 1);

    // generate some random leaves
    std::vector<NT::fr> leaves_frs;
    for (size_t l = 0; l < num_nonzero_leaves; l++) {
        leaves_frs.push_back(NT::fr::random_element());
    }
    // serilalize the leaves to a buffer to pass to cbind
    std::vector<uint8_t> leaves_bytes_vec;
    write(leaves_bytes_vec, leaves_frs);

    // setup output buffer
    // it must fit a uint32_t (for the vector length)
    // plus all of the nodes `frs` in the tree
    constexpr auto size_output_buf = sizeof(uint32_t) + (sizeof(NT::fr) * FUNCTION_TREE_NUM_NODES);
    std::array<uint8_t, size_output_buf> output = { 0 };

    // call cbind and get output (full tree root)
    abis__compute_function_tree(leaves_bytes_vec.data(), output.data());
    // deserialize output to vector of frs representing all nodes in tree
    std::vector<NT::fr> got_tree;
    uint8_t const* output_ptr = output.data();
    read(output_ptr, got_tree);

    // compare cbind results with direct computation

    // add the zero leaves to the vector of fields and pass to barretenberg helper
    NT::fr const zero_leaf = FunctionLeafPreimage<NT>().hash();  // hash of empty/0 preimage
    for (size_t l = num_nonzero_leaves; l < FUNCTION_TREE_NUM_LEAVES; l++) {
        leaves_frs.push_back(zero_leaf);
    }
    // compare results
    EXPECT_EQ(got_tree, plonk::stdlib::merkle_tree::compute_tree_native(leaves_frs));
}

TEST(abi_tests, hash_constructor)
{
    // Randomize required values
    auto const func_data = FunctionData<NT>{ .function_selector = 10, .is_private = true, .is_constructor = false };

    std::array<NT::fr, aztec3::ARGS_LENGTH> args;
    for (size_t i = 0; i < aztec3::ARGS_LENGTH; i++) {
        args[i] = fr::random_element();
    }
    NT::fr const constructor_vk_hash = NT::fr::random_element();

    // Write the function data and args to a buffer
    std::vector<uint8_t> func_data_buf;
    write(func_data_buf, func_data);

    std::vector<uint8_t> args_buf;
    write(args_buf, args);

    std::array<uint8_t, sizeof(NT::fr)> constructor_vk_hash_buf = { 0 };
    NT::fr::serialize_to_buffer(constructor_vk_hash, constructor_vk_hash_buf.data());

    // create an output buffer for cbind hash results
    std::array<uint8_t, sizeof(NT::fr)> output = { 0 };

    // Make the c_bind call to hash the constructor values
    abis__hash_constructor(func_data_buf.data(), args_buf.data(), constructor_vk_hash_buf.data(), output.data());

    // Convert buffer to `fr` for comparison to in-test calculated hash
    NT::fr const got_hash = NT::fr::serialize_from_buffer(output.data());

    // Calculate the expected hash in-test
    NT::fr const expected_hash = NT::compress(
        { func_data.hash(), NT::compress(args, aztec3::GeneratorIndex::CONSTRUCTOR_ARGS), constructor_vk_hash },
        aztec3::GeneratorIndex::CONSTRUCTOR);

    // Confirm cbind output == expected hash
    EXPECT_EQ(got_hash, expected_hash);
}

TEST(abi_tests, compute_contract_leaf)
{
    // Construct ContractLeafPreimage with some randomized fields
    NewContractData<NT> const preimage = NewContractData<NT>{
        .contract_address = NT::fr::random_element(),
        .portal_contract_address = NT::fr::random_element(),
        .function_tree_root = NT::fr::random_element(),
    };

    // Write the leaf preimage to a buffer
    std::vector<uint8_t> preimage_buf;
    write(preimage_buf, preimage);

    std::array<uint8_t, sizeof(NT::fr)> output = { 0 };
    abis__compute_contract_leaf(preimage_buf.data(), output.data());

    NT::fr const got_leaf = NT::fr::serialize_from_buffer(output.data());
    EXPECT_EQ(got_leaf, preimage.hash());
}

TEST(abi_tests, compute_transaction_hash)
{
    // randomize function args for tx request
    std::array<fr, ARGS_LENGTH> args;
    for (size_t i = 0; i < ARGS_LENGTH; i++) {
        args[i] = NT::fr::random_element();
    }

    // Construct TxRequest with some randomized fields
    TxRequest<NT> const tx_request = TxRequest<NT>{
        .from = NT::fr::random_element(),
        .to = NT::fr::random_element(),
        .function_data = FunctionData<NT>(),
        .args = args,
        .nonce = NT::fr::random_element(),
        .tx_context = TxContext<NT>(),
        .chain_id = NT::fr::random_element(),
    };

    std::array<uint8_t, 32> const r{
        0xf3, 0xac, 0x80, 0x61, 0xb5, 0x14, 0x79, 0x5b, 0x88, 0x43, 0xe3, 0xd6, 0x62, 0x95, 0x27, 0xed,
        0x2a, 0xfd, 0x6b, 0x1f, 0x6a, 0x55, 0x5a, 0x7a, 0xca, 0xbb, 0x5e, 0x6f, 0x79, 0xc8, 0xc2, 0xac,
    };

    std::array<uint8_t, 32> const s{
        0x8b, 0xf7, 0x78, 0x19, 0xca, 0x05, 0xa6, 0xb2, 0x78, 0x6c, 0x76, 0x26, 0x2b, 0xf7, 0x37, 0x1c,
        0xef, 0x97, 0xb2, 0x18, 0xe9, 0x6f, 0x17, 0x5a, 0x3c, 0xcd, 0xda, 0x2a, 0xcc, 0x05, 0x89, 0x03,
    };

    NT::ecdsa_signature const sig{ r, s, 27 };

    // Construct SignedTxRequest with some randomized fields
    SignedTxRequest<NT> const preimage = SignedTxRequest<NT>{
        .tx_request = tx_request,
        .signature = sig,
    };

    // Write the leaf preimage to a buffer
    std::vector<uint8_t> preimage_buf;
    write(preimage_buf, preimage);

    std::array<uint8_t, sizeof(NT::fr)> output = { 0 };
    abis__compute_transaction_hash(preimage_buf.data(), output.data());

    NT::fr const got_tx_hash = NT::fr::serialize_from_buffer(output.data());
    EXPECT_EQ(got_tx_hash, preimage.hash());
}

}  // namespace aztec3::circuits::abis
