#pragma once
#include "../bigfield/bigfield.hpp"
#include "../biggroup/biggroup.hpp"
#include "../field/field.hpp"
#include "barretenberg/ecc/curves/types.hpp"

namespace bb::stdlib {

template <typename CircuitBuilder> struct bn254 {
    static constexpr bb::CurveType type = bb::CurveType::BN254;
    // TODO(#673): This flag is temporary. It is needed in the verifier classes (GeminiVerifier, etc.) while these
    // classes are instantiated with "native" curve types. Eventually, the verifier classes will be instantiated only
    // with stdlib types, and "native" verification will be acheived via a simulated builder.
    static constexpr bool is_stdlib_type = true;

    // Corresponding native types (used exclusively for testing)
    using ScalarFieldNative = curve::BN254::ScalarField;
    using BaseFieldNative = curve::BN254::BaseField;
    using GroupNative = curve::BN254::Group;

    // Stdlib types corresponding to those defined in the native description of the curve.
    // Note: its useful to have these type names match the native analog exactly so that components that digest a Curve
    // (e.g. Gemini) can be agnostic as to whether they're operating on native or stdlib types.
    using ScalarField = field_t<CircuitBuilder>;
    using BaseField = bigfield<CircuitBuilder, bb::Bn254FqParams>;
    using Group = element<CircuitBuilder, BaseField, ScalarField, GroupNative>;
    using Element = Group;
    using AffineElement = Group;

    // Additional types with no analog in the native description of the curve
    using Builder = CircuitBuilder;
    using witness_ct = witness_t<CircuitBuilder>;
    using public_witness_ct = public_witness_t<CircuitBuilder>;
    using byte_array_ct = byte_array<CircuitBuilder>;
    using bool_ct = bool_t<CircuitBuilder>;
    using uint32_ct = stdlib::uint32<CircuitBuilder>;

    using bigfr_ct = bigfield<CircuitBuilder, bb::Bn254FrParams>;
    using g1_bigfr_ct = element<CircuitBuilder, BaseField, bigfr_ct, GroupNative>;

}; // namespace bn254
} // namespace bb::stdlib
