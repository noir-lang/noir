#pragma once

#include "../bigfield/bigfield.hpp"
#include "../biggroup/biggroup.hpp"
#include "../field/field.hpp"

#include <ecc/curves/secp256r1/secp256r1.hpp>

namespace plonk {
namespace stdlib {

template <typename ComposerType> struct secp256r1 {
    static constexpr waffle::CurveType type = waffle::CurveType::SECP256R1;

    typedef ::secp256r1::fq fq;
    typedef ::secp256r1::fr fr;
    typedef ::secp256r1::g1 g1;

    typedef ComposerType Composer;
    typedef witness_t<Composer> witness_ct;
    typedef public_witness_t<Composer> public_witness_ct;
    typedef field_t<Composer> fr_ct;
    typedef byte_array<Composer> byte_array_ct;
    typedef bool_t<Composer> bool_ct;
    typedef stdlib::uint32<Composer> uint32_ct;

    typedef bigfield<Composer, typename ::secp256r1::Secp256r1FqParams> fq_ct;
    typedef bigfield<Composer, typename ::secp256r1::Secp256r1FrParams> bigfr_ct;
    typedef element<Composer, fq_ct, fr_ct, g1> g1_ct;
    typedef element<Composer, fq_ct, bigfr_ct, g1> g1_bigfr_ct;
};
} // namespace stdlib
} // namespace plonk