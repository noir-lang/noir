#pragma once
#include <ecc/curves/types.hpp>
#include "../bigfield/bigfield.hpp"
#include "../biggroup/biggroup.hpp"
#include "../field/field.hpp"

namespace plonk {
namespace stdlib {

template <typename ComposerType> struct bn254 {
    static constexpr waffle::CurveType type = waffle::CurveType::BN254;

    typedef barretenberg::fq fq;
    typedef barretenberg::fr fr;
    typedef barretenberg::g1 g1;

    typedef ComposerType Composer;
    typedef witness_t<Composer> witness_ct;
    typedef public_witness_t<Composer> public_witness_ct;
    typedef field_t<Composer> fr_ct;
    typedef byte_array<Composer> byte_array_ct;
    typedef bool_t<Composer> bool_ct;
    typedef stdlib::uint32<Composer> uint32_ct;

    typedef bigfield<Composer, barretenberg::Bn254FqParams> fq_ct;
    typedef bigfield<Composer, barretenberg::Bn254FrParams> bigfr_ct;
    typedef element<Composer, fq_ct, fr_ct, g1> g1_ct;
    typedef element<Composer, fq_ct, bigfr_ct, g1> g1_bigfr_ct;

}; // namespace bn254
} // namespace stdlib
} // namespace plonk