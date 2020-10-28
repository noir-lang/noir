#pragma once
#include <common/net.hpp>
#include <crypto/blake2s/blake2s.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <stdlib/hash/blake2s/blake2s.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/primitives/field/field.hpp>
#include <vector>

namespace plonk {
namespace stdlib {
namespace merkle_tree {

template <typename ComposerContext> inline field_t<ComposerContext> hash_value(byte_array<ComposerContext> const& input, const bool use_blake2s = false)
{
    ASSERT(input.get_context() != nullptr);
    if (use_blake2s)
    {
        const auto result = static_cast<field_t<ComposerContext>>(stdlib::blake2s(input));
        return result;
    }
    else
    {
        const auto result = static_cast<field_t<ComposerContext>>(plonk::stdlib::pedersen<ComposerContext>::compress(input));
        return result;
    }
}

inline barretenberg::fr hash_value_native(std::vector<uint8_t> const& input, const bool use_blake2s = false)
{
    std::vector<uint8_t> output;
    if (use_blake2s)
    {
        output = blake2::blake2s(input);
    }
    else
    {
        output = crypto::pedersen::compress_native(input);
    }
    return barretenberg::fr::serialize_from_buffer(output.data());
}

inline barretenberg::fr compress_native(std::vector<barretenberg::fr> const& input)
{
    return crypto::pedersen::compress_native(input[0], input[1]);
}

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk