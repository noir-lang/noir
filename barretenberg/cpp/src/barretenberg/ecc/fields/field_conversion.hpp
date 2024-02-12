#pragma once

#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/proof_system/types/circuit_type.hpp"

namespace bb::field_conversion {

/**
 * @brief Calculates the size of a types in terms of bb::frs
 * @details We want to support the following types: bool, size_t, uint32_t, uint64_t, bb::fr, grumpkin::fr,
 * curve::BN254::AffineElement, curve::Grumpkin::AffineElement, bb::Univariate<FF, N>, std::array<FF, N>, for
 * FF = bb::fr/grumpkin::fr, and N is arbitrary
 * @tparam T
 * @return constexpr size_t
 */
template <typename T> constexpr size_t calc_num_bn254_frs()
{
    if constexpr (IsAnyOf<T, uint32_t, bool>) {
        return 1;
    } else if constexpr (IsAnyOf<T, bb::fr, grumpkin::fr>) {
        return T::Params::NUM_BN254_SCALARS;
    } else if constexpr (IsAnyOf<T, curve::BN254::AffineElement, curve::Grumpkin::AffineElement>) {
        return 2 * calc_num_bn254_frs<typename T::Fq>();
    } else {
        // Array or Univariate
        return calc_num_bn254_frs<typename T::value_type>() * (std::tuple_size<T>::value);
    }
}

grumpkin::fr convert_grumpkin_fr_from_bn254_frs(std::span<const bb::fr> fr_vec);

/**
 * @brief Conversions from vector of bb::fr elements to transcript types.
 * @details We want to support the following types: bool, size_t, uint32_t, uint64_t, bb::fr, grumpkin::fr,
 * curve::BN254::AffineElement, curve::Grumpkin::AffineElement, bb::Univariate<FF, N>, std::array<FF, N>, for
 * FF = bb::fr/grumpkin::fr, and N is arbitrary.
 * The only nontrivial implementation is the conversion for grumpkin::fr. More details are given in the function comment
 * below.
 * @tparam T
 * @param fr_vec
 * @return T
 */
template <typename T> T convert_from_bn254_frs(std::span<const bb::fr> fr_vec)
{
    if constexpr (IsAnyOf<T, bool>) {
        ASSERT(fr_vec.size() == 1);
        return bool(fr_vec[0]);
    } else if constexpr (IsAnyOf<T, uint32_t, bb::fr>) {
        ASSERT(fr_vec.size() == 1);
        return static_cast<T>(fr_vec[0]);
    } else if constexpr (IsAnyOf<T, grumpkin::fr>) {
        ASSERT(fr_vec.size() == 2);
        return convert_grumpkin_fr_from_bn254_frs(fr_vec);
    } else if constexpr (IsAnyOf<T, curve::BN254::AffineElement, curve::Grumpkin::AffineElement>) {
        using BaseField = typename T::Fq;
        constexpr size_t BASE_FIELD_SCALAR_SIZE = calc_num_bn254_frs<BaseField>();
        ASSERT(fr_vec.size() == 2 * BASE_FIELD_SCALAR_SIZE);
        T val;
        val.x = convert_from_bn254_frs<BaseField>(fr_vec.subspan(0, BASE_FIELD_SCALAR_SIZE));
        val.y = convert_from_bn254_frs<BaseField>(fr_vec.subspan(BASE_FIELD_SCALAR_SIZE, BASE_FIELD_SCALAR_SIZE));
        return val;
    } else {
        // Array or Univariate
        T val;
        constexpr size_t FieldScalarSize = calc_num_bn254_frs<typename T::value_type>();
        ASSERT(fr_vec.size() == FieldScalarSize * std::tuple_size<T>::value);
        size_t i = 0;
        for (auto& x : val) {
            x = convert_from_bn254_frs<typename T::value_type>(fr_vec.subspan(FieldScalarSize * i, FieldScalarSize));
            ++i;
        }
        return val;
    }
}

std::vector<bb::fr> convert_grumpkin_fr_to_bn254_frs(const grumpkin::fr& val);

/**
 * @brief Conversion from transcript values to bb::frs
 * @details We want to support the following types: bool, size_t, uint32_t, uint64_t, bb::fr, grumpkin::fr,
 * curve::BN254::AffineElement, curve::Grumpkin::AffineElement, bb::Univariate<FF, N>, std::array<FF, N>, for
 * FF = bb::fr/grumpkin::fr, and N is arbitrary.
 * @tparam T
 * @param val
 * @return std::vector<bb::fr>
 */
template <typename T> std::vector<bb::fr> convert_to_bn254_frs(const T& val)
{
    if constexpr (IsAnyOf<T, bool, uint32_t, bb::fr>) {
        std::vector<bb::fr> fr_vec{ val };
        return fr_vec;
    } else if constexpr (IsAnyOf<T, grumpkin::fr>) {
        return convert_grumpkin_fr_to_bn254_frs(val);
    } else if constexpr (IsAnyOf<T, curve::BN254::AffineElement, curve::Grumpkin::AffineElement>) {
        auto fr_vec_x = convert_to_bn254_frs(val.x);
        auto fr_vec_y = convert_to_bn254_frs(val.y);
        std::vector<bb::fr> fr_vec(fr_vec_x.begin(), fr_vec_x.end());
        fr_vec.insert(fr_vec.end(), fr_vec_y.begin(), fr_vec_y.end());
        return fr_vec;
    } else {
        // Array or Univariate
        std::vector<bb::fr> fr_vec;
        for (auto& x : val) {
            auto tmp_vec = convert_to_bn254_frs(x);
            fr_vec.insert(fr_vec.end(), tmp_vec.begin(), tmp_vec.end());
        }
        return fr_vec;
    }
}

grumpkin::fr convert_to_grumpkin_fr(const bb::fr& f);

template <typename T> T inline convert_challenge(const bb::fr& challenge)
{
    if constexpr (std::is_same_v<T, bb::fr>) {
        return challenge;
    } else if constexpr (std::is_same_v<T, grumpkin::fr>) {
        return convert_to_grumpkin_fr(challenge);
    }
}

} // namespace bb::field_conversion
