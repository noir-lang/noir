#pragma once

#include "../bigfield/bigfield.hpp"
#include "../biggroup/biggroup.hpp"
#include "../field/field.hpp"

#include "barretenberg/ecc/curves/secp256r1/secp256r1.hpp"

namespace bb::stdlib {

template <typename CircuitType> struct secp256r1 {
    static constexpr bb::CurveType type = bb::CurveType::SECP256R1;

    typedef ::secp256r1::fq fq;
    typedef ::secp256r1::fr fr;
    typedef ::secp256r1::g1 g1;

    typedef CircuitType Builder;
    typedef witness_t<Builder> witness_ct;
    typedef public_witness_t<Builder> public_witness_ct;
    typedef field_t<Builder> fr_ct;
    typedef byte_array<Builder> byte_array_ct;
    typedef bool_t<Builder> bool_ct;
    typedef stdlib::uint32<Builder> uint32_ct;

    typedef bigfield<Builder, typename ::secp256r1::Secp256r1FqParams> fq_ct;
    typedef bigfield<Builder, typename ::secp256r1::Secp256r1FrParams> bigfr_ct;
    typedef element<Builder, fq_ct, fr_ct, g1> g1_ct;
    typedef element<Builder, fq_ct, bigfr_ct, g1> g1_bigfr_ct;
};
} // namespace bb::stdlib
