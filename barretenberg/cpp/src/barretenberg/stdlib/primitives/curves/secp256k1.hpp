#pragma once

#include "../bigfield/bigfield.hpp"
#include "../biggroup/biggroup.hpp"
#include "../field/field.hpp"

#include "barretenberg/ecc/curves/secp256k1/secp256k1.hpp"

namespace bb::stdlib {

template <typename CircuitType> struct secp256k1 {
    static constexpr bb::CurveType type = bb::CurveType::SECP256K1;

    typedef ::secp256k1::fq fq;
    typedef ::secp256k1::fr fr;
    typedef ::secp256k1::g1 g1;

    typedef CircuitType Builder;
    typedef witness_t<Builder> witness_ct;
    typedef public_witness_t<Builder> public_witness_ct;
    typedef field_t<Builder> fr_ct;
    typedef byte_array<Builder> byte_array_ct;
    typedef bool_t<Builder> bool_ct;
    typedef stdlib::uint32<Builder> uint32_ct;

    typedef bigfield<Builder, typename ::secp256k1::Secp256k1FqParams> fq_ct;
    typedef bigfield<Builder, typename ::secp256k1::Secp256k1FrParams> bigfr_ct;
    typedef element<Builder, fq_ct, fr_ct, g1> g1_ct;
    typedef element<Builder, fq_ct, bigfr_ct, g1> g1_bigfr_ct;
};
} // namespace bb::stdlib
