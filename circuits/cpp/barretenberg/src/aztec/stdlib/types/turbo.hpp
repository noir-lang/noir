#pragma once
#include <plonk/composer/turbo_composer.hpp>
#include <stdlib/primitives/bigfield/bigfield.hpp>
#include <stdlib/primitives/biggroup/biggroup.hpp>
#include <stdlib/primitives/bit_array/bit_array.hpp>
#include <stdlib/primitives/bool/bool.hpp>
#include <stdlib/primitives/byte_array/byte_array.hpp>
#include <stdlib/primitives/uint/uint.hpp>
#include <stdlib/primitives/witness/witness.hpp>

namespace plonk {
namespace stdlib {
namespace types {
namespace turbo {

using namespace plonk;

typedef waffle::TurboComposer Composer;
typedef waffle::TurboProver Prover;
typedef waffle::UnrolledTurboProver UnrolledProver;
typedef waffle::TurboVerifier Verifier;
typedef waffle::UnrolledTurboVerifier UnrolledVerifier;
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
typedef stdlib::bigfield<Composer, barretenberg::Bn254FqParams> fq_ct;
typedef stdlib::element<Composer, fq_ct, field_ct, barretenberg::g1> group_ct;

struct point {
    field_ct x;
    field_ct y;
};

} // namespace turbo
} // namespace types
} // namespace stdlib
} // namespace plonk
