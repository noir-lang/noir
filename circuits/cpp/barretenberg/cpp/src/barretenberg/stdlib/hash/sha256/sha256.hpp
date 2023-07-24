#pragma once
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders_fwd.hpp"
#include "barretenberg/stdlib/primitives/packed_byte_array/packed_byte_array.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include "sha256_plookup.hpp"
#include <array>
// namespace proof_system::plonk

namespace proof_system::plonk {
namespace stdlib {
template <typename Composer> class bit_array;

template <typename Composer>
std::array<uint32<Composer>, 8> sha256_block(const std::array<uint32<Composer>, 8>& h_init,
                                             const std::array<uint32<Composer>, 16>& input);

template <typename Composer> byte_array<Composer> sha256_block(const byte_array<Composer>& input);
template <typename Composer> packed_byte_array<Composer> sha256(const packed_byte_array<Composer>& input);

template <typename Composer> field_t<Composer> sha256_to_field(const packed_byte_array<Composer>& input)
{
    std::vector<field_t<Composer>> slices = stdlib::sha256<Composer>(input).to_unverified_byte_slices(16);
    return slices[1] + (slices[0] * (uint256_t(1) << 128));
}

#define SHA256_BLOCK(circuit_type) byte_array<circuit_type> sha256_block(const byte_array<circuit_type>& input)
#define SHA256(circuit_type) packed_byte_array<circuit_type> sha256(const packed_byte_array<circuit_type>& input)

EXTERN_STDLIB_METHOD(SHA256_BLOCK)
EXTERN_STDLIB_METHOD(SHA256)

} // namespace stdlib
} // namespace proof_system::plonk
