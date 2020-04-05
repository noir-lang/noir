#pragma once
#include <plonk/composer/standard_composer.hpp>
#include <stdlib/primitives/bit_array/bit_array.hpp>
#include <stdlib/primitives/bool/bool.hpp>
#include <stdlib/primitives/byte_array/byte_array.hpp>
#include <stdlib/primitives/uint/uint.hpp>
#include <stdlib/primitives/witness/witness.hpp>

namespace plonk {
namespace stdlib {
namespace types {
namespace standard {

using namespace plonk;

typedef waffle::StandardComposer Composer;
typedef waffle::Prover Prover;
typedef waffle::Verifier Verifier;
typedef stdlib::witness_t<Composer> witness_ct;
typedef stdlib::public_witness_t<Composer> public_witness_ct;
typedef stdlib::bool_t<Composer> bool_ct;
typedef stdlib::byte_array<Composer> byte_array_ct;
typedef stdlib::field_t<Composer> field_ct;
typedef stdlib::uint8<Composer> uint8_ct;
typedef stdlib::uint16<Composer> uint16_ct;
typedef stdlib::uint32<Composer> uint32_ct;
typedef stdlib::uint64<Composer> uint64_ct;
typedef stdlib::bit_array<Composer> bit_array_ct;

struct point {
    field_ct x;
    field_ct y;
};

} // namespace standard
} // namespace types
} // namespace stdlib
} // namespace plonk