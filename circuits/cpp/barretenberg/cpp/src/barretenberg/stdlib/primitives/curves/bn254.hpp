#pragma once
#include "../bigfield/bigfield.hpp"
#include "../biggroup/biggroup.hpp"
#include "../field/field.hpp"
#include "barretenberg/ecc/curves/types.hpp"

namespace proof_system::plonk {
namespace stdlib {

template <typename CircuitBuilder> struct bn254 {
    static constexpr proof_system::CurveType type = proof_system::CurveType::BN254;

    // NOTE: Naming in flux here; maybe name should reflect "native" somehow?
    using BaseField = curve::BN254::BaseField;
    using fq = BaseField;
    using ScalarField = curve::BN254::ScalarField;
    using fr = ScalarField;
    using Group = curve::BN254::Group;
    using g1 = Group;

    using Builder = CircuitBuilder;
    using Composer = CircuitBuilder;
    typedef witness_t<CircuitBuilder> witness_ct;
    typedef public_witness_t<CircuitBuilder> public_witness_ct;
    typedef field_t<CircuitBuilder> fr_ct;
    typedef byte_array<CircuitBuilder> byte_array_ct;
    typedef bool_t<CircuitBuilder> bool_ct;
    typedef stdlib::uint32<CircuitBuilder> uint32_ct;

    typedef bigfield<CircuitBuilder, barretenberg::Bn254FqParams> fq_ct;
    typedef bigfield<CircuitBuilder, barretenberg::Bn254FrParams> bigfr_ct;
    typedef element<CircuitBuilder, fq_ct, fr_ct, Group> g1_ct;
    typedef element<CircuitBuilder, fq_ct, bigfr_ct, Group> g1_bigfr_ct;

}; // namespace bn254
} // namespace stdlib
} // namespace proof_system::plonk
