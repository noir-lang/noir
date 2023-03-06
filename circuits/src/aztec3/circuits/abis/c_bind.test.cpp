#include "c_bind.h"

#include "tx_request.hpp"

#include <numeric/random/engine.hpp>
#include <gtest/gtest.h>

namespace {
using NT = plonk::stdlib::types::NativeTypes;
auto& engine = numeric::random::get_debug_engine();
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
    uint8_t* output = (uint8_t*)malloc(32 * sizeof(uint8_t));
    // Make the c_bind call to hash the tx request
    abis__hash_tx_request(buf.data(), output);

    // Convert buffer to `fr` for comparison to in-test calculated hash
    NT::fr got_hash = NT::fr::serialize_from_buffer(output);
    // Confirm cbind output == hash of tx request
    EXPECT_EQ(got_hash, tx_request.hash());
}

} // namespace aztec3::circuits::abis
