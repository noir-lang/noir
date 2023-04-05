#pragma once
#include "barretenberg/common/net.hpp"
#include "barretenberg/crypto/blake2s/blake2s.hpp"
#include "barretenberg/crypto/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/hash/blake2s/blake2s.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include <vector>

namespace proof_system::plonk {
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
} // namespace proof_system::plonk
