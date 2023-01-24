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

template <typename ComposerContext> inline field_t<ComposerContext> hash_value(byte_array<ComposerContext> const& input)
{
    return plonk::stdlib::pedersen<ComposerContext>::compress(input);
}

inline barretenberg::fr hash_value_native(std::vector<uint8_t> const& input)
{
    return crypto::pedersen::compress_native(input);
}

inline barretenberg::fr compress_native(barretenberg::fr const& lhs, barretenberg::fr const& rhs)
{
    return crypto::pedersen::compress_native({ lhs, rhs });
}

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk