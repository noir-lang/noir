#include "c_bind.h"

#include "tx_request.hpp"

#include <numeric/random/engine.hpp>
#include <gtest/gtest.h>

namespace {

using NT = plonk::stdlib::types::NativeTypes;
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

TEST(abis, hash_tx_request)
{
    // randomize function args for tx request
    std::array<fr, ARGS_LENGTH> args;
    for (size_t i = 0; i < ARGS_LENGTH; i++) {
        args[i] = fr(engine.get_random_uint256());
    }

    // Construct mostly empty TxRequest with some randomized fields
    TxRequest<NT> tx_request = TxRequest<NT>{
        .from = engine.get_random_uint256(),
        .to = engine.get_random_uint256(),
        .function_signature = FunctionSignature<NT>(),
        .args = args,
        .nonce = engine.get_random_uint256(),
        .tx_context = TxContext<NT>(),
        .chain_id = engine.get_random_uint256(),
    };

    // Write the tx request to a buffer and
    // allocate an output buffer for cbind hash results
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

TEST(abis, compute_function_selector_transfer)
{
    const char* function_signature = "transfer(address,uint256)";

    // create an output buffer for cbind selector results
    std::array<uint8_t, 4> output = { 0 };
    // Make the c_bind call to compute the function selector via keccak256
    abis__compute_function_selector(function_signature, output.data());

    // get the selector as a hex string of 4 bytes and
    // compare against known good selector from solidity
    EXPECT_EQ(bytes_to_hex_str(output), "a9059cbb");
}

TEST(abis, compute_function_selector_transferFrom)
{
    const char* function_signature = "transferFrom(address,address,uint256)";

    // create an output buffer for cbind selector results
    std::array<uint8_t, 4> output = { 0 };
    // Make the c_bind call to compute the function selector via keccak256
    abis__compute_function_selector(function_signature, output.data());

    // get the selector as a hex string of 4 bytes and
    // compare against known good selector from solidity
    EXPECT_EQ(bytes_to_hex_str(output), "23b872dd");
}

} // namespace aztec3::circuits::abis
