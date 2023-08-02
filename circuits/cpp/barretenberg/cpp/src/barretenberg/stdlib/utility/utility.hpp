#pragma once

#include "barretenberg/ecc/curves/bn254/fq.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/honk/sumcheck/polynomials/univariate.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"

#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"
#include "barretenberg/stdlib/primitives/biggroup/biggroup.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"

namespace proof_system::plonk::stdlib::recursion::utility {

/**
 * @brief Utility class for converting native types to corresponding stdlib types
 *
 * @details Used to facilitate conversion of various native types (uint32_t, field, group, Univarite, etc.) to
 * corresponding stdlib types. Useful for example for obtaining stdlib types in the recursive trancript from native
 * types upon deserialization from the native transcript.
 *
 * @todo Eliminate the need for these somehow?
 * @tparam Builder
 */
template <typename Builder> class StdlibTypesUtility {
    using field_ct = field_t<Builder>;
    using witness_ct = witness_t<Builder>;
    using fq_ct = bigfield<Builder, barretenberg::Bn254FqParams>;
    using element_ct = element<Builder, fq_ct, field_ct, barretenberg::g1>;
    using FF = barretenberg::fr;
    using Commitment = barretenberg::g1::affine_element;
    template <size_t LENGTH> using Univariate = proof_system::honk::sumcheck::Univariate<FF, LENGTH>;

  public:
    /**
     * @brief Construct stdlib field from uint32_t
     *
     * @param element
     * @return field_ct
     */
    static field_ct from_witness(Builder* builder, uint32_t native_element)
    {
        return field_ct::from_witness(builder, native_element);
    }

    /**
     * @brief Construct stdlib field from native field type
     *
     * @param native_element
     * @return field_ct
     */
    static field_ct from_witness(Builder* builder, FF native_element)
    {
        return field_ct::from_witness(builder, native_element);
    }

    /**
     * @brief Construct stdlib group from native affine group element type
     *
     * @param native_element
     * @return field_ct
     */
    static element_ct from_witness(Builder* builder, Commitment native_element)
    {
        return element_ct::from_witness(builder, native_element);
    }

    /**
     * @brief Construct field_t array from native field array
     * @param native_element Array of FF
     * @return std::array<field_ct, LENGTH>
     */
    template <size_t LENGTH>
    static std::array<field_ct, LENGTH> from_witness(Builder* builder, std::array<FF, LENGTH> native_element)
    {
        std::array<field_ct, LENGTH> element;
        for (size_t i = 0; i < LENGTH; ++i) {
            element[i] = field_ct::from_witness(builder, native_element[i]);
        }
        return element;
    }

    /**
     * @brief Construct field_t array from native Univariate type
     * TODO(luke): do we need a stdlib Univariate or is std::array<field_t> good enough?
     * @param native_element
     * @return std::array<field_ct, LENGTH>
     */
    template <size_t LENGTH>
    static std::array<field_ct, LENGTH> from_witness(Builder* builder, Univariate<LENGTH> native_element)
    {
        std::array<field_ct, LENGTH> element;
        for (size_t i = 0; i < LENGTH; ++i) {
            element[i] = field_ct::from_witness(builder, native_element.value_at(i));
        }
        return element;
    }
};
} // namespace proof_system::plonk::stdlib::recursion::utility