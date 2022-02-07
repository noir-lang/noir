#pragma once
#include <plonk/composer/standard_composer.hpp>
#include <stdlib/primitives/bigfield/bigfield.hpp>
#include <stdlib/primitives/biggroup/biggroup.hpp>
#include <stdlib/primitives/bit_array/bit_array.hpp>
#include <stdlib/primitives/bool/bool.hpp>
#include <stdlib/primitives/byte_array/byte_array.hpp>
#include <stdlib/primitives/packed_byte_array/packed_byte_array.hpp>
#include <stdlib/primitives/uint/uint.hpp>
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/primitives/point/point.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/merkle_tree/hash_path.hpp>
#include <stdlib/encryption/schnorr/schnorr.hpp>
#include <stdlib/primitives/curves/bn254.hpp>
#include <stdlib/primitives/group/group.hpp>
#include <stdlib/recursion/verifier/verifier.hpp>

namespace plonk {
namespace stdlib {
namespace types {
namespace standard {

using namespace plonk;

typedef waffle::StandardComposer Composer;
typedef waffle::Prover Prover;
typedef waffle::UnrolledProver UnrolledProver;
typedef waffle::Verifier Verifier;
typedef stdlib::witness_t<Composer> witness_ct;
typedef stdlib::public_witness_t<Composer> public_witness_ct;
typedef stdlib::bool_t<Composer> bool_ct;
typedef stdlib::byte_array<Composer> byte_array_ct;
typedef stdlib::packed_byte_array<Composer> packed_byte_array_ct;
typedef stdlib::field_t<Composer> field_ct;
typedef stdlib::uint8<Composer> uint8_ct;
typedef stdlib::uint16<Composer> uint16_ct;
typedef stdlib::uint32<Composer> uint32_ct;
typedef stdlib::uint64<Composer> uint64_ct;
typedef stdlib::bit_array<Composer> bit_array_ct;
typedef stdlib::bigfield<Composer, barretenberg::Bn254FqParams> fq_ct;
typedef stdlib::element<Composer, fq_ct, field_ct, barretenberg::g1> biggroup_ct;
typedef stdlib::point<Composer> point_ct;
typedef stdlib::pedersen<Composer> pedersen;
typedef stdlib::group<Composer> group_ct;

typedef stdlib::bn254<Composer> bn254;

// these are used in biggroup tests
typedef stdlib::bn254<Composer> bn254_1;
typedef stdlib::bn254<Composer> bn254_2;
typedef stdlib::bigfield<Composer, barretenberg::Bn254FrParams> big_fr_ct;
typedef stdlib::element<Composer, fq_ct, big_fr_ct, barretenberg::g1> bigfield_biggroup_ct;

namespace merkle_tree {
using namespace stdlib::merkle_tree;
typedef stdlib::merkle_tree::hash_path<Composer> hash_path;
} // namespace merkle_tree

namespace schnorr {
typedef stdlib::schnorr::signature_bits<Composer> signature_bits;
}
} // namespace standard
} // namespace types
} // namespace stdlib
} // namespace plonk