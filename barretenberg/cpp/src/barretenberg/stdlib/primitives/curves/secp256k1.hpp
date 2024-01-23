#pragma once

#include "../bigfield/bigfield.hpp"
#include "../biggroup/biggroup.hpp"
#include "../field/field.hpp"

#include "barretenberg/ecc/curves/secp256k1/secp256k1.hpp"

namespace bb::stdlib {

template <typename CircuitType> struct secp256k1 {
    static constexpr bb::CurveType type = bb::CurveType::SECP256K1;

    using fq = ::secp256k1::fq;
    using fr = ::secp256k1::fr;
    using g1 = ::secp256k1::g1;

    using Builder = CircuitType;
    using witness_ct = witness_t<Builder>;
    using public_witness_ct = public_witness_t<Builder>;
    using fr_ct = field_t<Builder>;
    using byte_array_ct = byte_array<Builder>;
    using bool_ct = bool_t<Builder>;
    using uint32_ct = stdlib::uint32<Builder>;

    using fq_ct = bigfield<Builder, typename ::secp256k1::Secp256k1FqParams>;
    using bigfr_ct = bigfield<Builder, typename ::secp256k1::Secp256k1FrParams>;
    using g1_ct = element<Builder, fq_ct, fr_ct, g1>;
    using g1_bigfr_ct = element<Builder, fq_ct, bigfr_ct, g1>;
};
} // namespace bb::stdlib
