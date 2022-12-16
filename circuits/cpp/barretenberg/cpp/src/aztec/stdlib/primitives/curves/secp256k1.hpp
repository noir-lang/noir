#pragma once

#include "../bigfield/bigfield.hpp"
#include "../biggroup/biggroup.hpp"
#include "../field/field.hpp"

#include <ecc/curves/secp256k1/secp256k1.hpp>

namespace plonk {
namespace stdlib {

template <typename ComposerType> struct secp256k1 {
    static constexpr waffle::CurveType type = waffle::CurveType::SECP256K1;

    typedef ::secp256k1::fq fq;
    typedef ::secp256k1::fr fr;
    typedef ::secp256k1::g1 g1;

    typedef ComposerType Composer;
    typedef witness_t<Composer> witness_ct;
    typedef public_witness_t<Composer> public_witness_ct;
    typedef field_t<Composer> fr_ct;
    typedef byte_array<Composer> byte_array_ct;
    typedef bool_t<Composer> bool_ct;
    typedef stdlib::uint32<Composer> uint32_ct;

    typedef bigfield<Composer, typename ::secp256k1::Secp256k1FqParams> fq_ct;
    typedef bigfield<Composer, typename ::secp256k1::Secp256k1FrParams> bigfr_ct;
    typedef element<Composer, fq_ct, fr_ct, g1> g1_ct;
    typedef element<Composer, fq_ct, bigfr_ct, g1> g1_bigfr_ct;
};
} // namespace stdlib
} // namespace plonk