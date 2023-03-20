#include "c_bind.h"

#include "tx_request.hpp"
#include "function_leaf_preimage.hpp"

#include <barretenberg/numeric/random/engine.hpp>
#include <gtest/gtest.h>

namespace {

using NT = aztec3::utils::types::NativeTypes;
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

} // namespace

namespace aztec3::circuits::abis {

TEST(abi_tests, hash_tx_request)
{
    // randomize function args for tx request
    std::array<fr, ARGS_LENGTH> args;
    for (size_t i = 0; i < ARGS_LENGTH; i++) {
        args[i] = fr(engine.get_random_uint256());
    }

    // Construct TxRequest with some randomized fields
    TxRequest<NT> tx_request = TxRequest<NT>{
        .from = engine.get_random_uint256(),
        .to = engine.get_random_uint256(),
        .function_data = FunctionData<NT>(),
        .args = args,
        .nonce = engine.get_random_uint256(),
        .tx_context = TxContext<NT>(),
        .chain_id = engine.get_random_uint256(),
    };

    // Write the tx request to a buffer and
    std::vector<uint8_t> buf;
    write(buf, tx_request);

    // create an output buffer for cbind hash results
    std::array<uint8_t, 32> output = { 0 };
    // Make the c_bind call to hash the tx request
    abis__hash_tx_request(buf.data(), output.data());

    // Convert buffer to `fr` for comparison to in-test calculated hash
    NT::fr got_hash = NT::fr::serialize_from_buffer(output.data());

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
    std::string full_selector = "a9059cbb2ab09eb219583f4a59a5d0623ade346d962bcd4e46b11da047c9049b";
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
    std::string full_selector = "23b872dd7302113369cda2901243429419bec145408fa8b352b3dd92b66c680b";
    EXPECT_EQ(bytes_to_hex_str(output), full_selector.substr(0, FUNCTION_SELECTOR_NUM_BYTES * 2));
}

TEST(abi_tests, compute_function_leaf)
{
    // Construct FunctionLeafPreimage with some randomized fields
    FunctionLeafPreimage<NT> preimage = FunctionLeafPreimage<NT>{
        .function_selector = engine.get_random_uint256(),
        .is_private = static_cast<bool>(engine.get_random_uint8() & 1),
        .vk_hash = engine.get_random_uint256(),
        .acir_hash = engine.get_random_uint256(),
    };

    // Write the leaf preimage to a buffer
    std::vector<uint8_t> preimage_buf;
    write(preimage_buf, preimage);

    std::array<uint8_t, 32> output = { 0 };
    abis__compute_function_leaf(preimage_buf.data(), output.data());

    NT::fr got_leaf = NT::fr::serialize_from_buffer(output.data());
    EXPECT_EQ(got_leaf, preimage.hash());
}

} // namespace aztec3::circuits::abis
