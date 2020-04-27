#pragma once
#include <common/net.hpp>
#include <crypto/blake2s/blake2s.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <stdlib/hash/blake2s/blake2s.hpp>
#include <stdlib/primitives/field/field.hpp>
#include <vector>

namespace plonk {
namespace stdlib {
namespace merkle_tree {

template <typename ComposerContext> inline field_t<ComposerContext> hash_value(byte_array<ComposerContext> const& input)
{
    ASSERT(input.get_context() != nullptr);
    return static_cast<field_t<ComposerContext>>(stdlib::blake2s(input));
}

inline barretenberg::fr hash_value_native(std::string const& input)
{
    std::vector<uint8_t> inputv(input.begin(), input.end());
    std::vector<uint8_t> output = blake2::blake2s(inputv);
    return barretenberg::fr::serialize_from_buffer(output.data());
}

inline barretenberg::fr compress_native(std::vector<barretenberg::fr> const& input)
{
    return crypto::pedersen::compress_native(input[0], input[1]);
}

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk