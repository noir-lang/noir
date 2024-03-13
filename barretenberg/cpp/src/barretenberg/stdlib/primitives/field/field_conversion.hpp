#pragma once

#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/group/cycle_group.hpp"

namespace bb::stdlib::field_conversion {

template <typename Builder> using fr = field_t<Builder>;
template <typename Builder> using fq = bigfield<Builder, bb::Bn254FqParams>;
template <typename Builder> using bn254_element = element<Builder, fq<Builder>, fr<Builder>, curve::BN254::Group>;
template <typename Builder> using grumpkin_element = cycle_group<Builder>;

static constexpr uint64_t NUM_LIMB_BITS = NUM_LIMB_BITS_IN_FIELD_SIMULATION;
static constexpr uint64_t TOTAL_BITS = 254;

template <typename Builder> fq<Builder> convert_to_grumpkin_fr(Builder& builder, const fr<Builder>& f);

template <typename Builder, typename T> inline T convert_challenge(Builder& builder, const fr<Builder>& challenge)
{
    if constexpr (std::is_same_v<T, fr<Builder>>) {
        return challenge;
    } else if constexpr (std::is_same_v<T, fq<Builder>>) {
        return convert_to_grumpkin_fr(builder, challenge);
    }
}

template <typename Builder> inline std::vector<fr<Builder>> convert_grumpkin_fr_to_bn254_frs(const fq<Builder>& input)
{
    fr<Builder> shift(static_cast<uint256_t>(1) << NUM_LIMB_BITS);
    std::vector<fr<Builder>> result(2);
    result[0] = input.binary_basis_limbs[0].element + (input.binary_basis_limbs[1].element * shift);
    result[1] = input.binary_basis_limbs[2].element + (input.binary_basis_limbs[3].element * shift);
    return result;
}
/**
 * @brief Calculates the size of a types (in their native form) in terms of fr<Builder>s
 * @details We want to support the following types: fr<Builder>, fq<Builder>,
 * bn254_element<Builder>, grumpkin_element<Builder, bb::Univariate<FF, N>, std::array<FF, N>, for
 * FF = fr<Builder> or fq<Builder>, and N is arbitrary
 * @tparam Builder
 * @tparam T
 * @return constexpr size_t
 */
template <typename Builder, typename T> constexpr size_t calc_num_bn254_frs()
{
    if constexpr (IsAnyOf<T, fr<Builder>>) {
        return Bn254FrParams::NUM_BN254_SCALARS;
    } else if constexpr (IsAnyOf<T, fq<Builder>>) {
        return Bn254FqParams::NUM_BN254_SCALARS;
    } else if constexpr (IsAnyOf<T, bn254_element<Builder>>) {
        return 2 * calc_num_bn254_frs<Builder, fq<Builder>>();
    } else if constexpr (IsAnyOf<T, grumpkin_element<Builder>>) {
        return 2 * calc_num_bn254_frs<Builder, fr<Builder>>();
    } else {
        // Array or Univariate
        return calc_num_bn254_frs<Builder, typename T::value_type>() * (std::tuple_size<T>::value);
    }
}

/**
 * @brief Conversions from vector of fr<Builder> elements to transcript types.
 * @details We want to support the following types: fr<Builder>, fq<Builder>,
 * bn254_element<Builder>, grumpkin_element<Builder, bb::Univariate<FF, N>, std::array<FF, N>, for
 * FF = fr<Builder> or fq<Builder>, and N is arbitrary
 * @tparam Builder
 * @tparam T
 * @param builder
 * @param fr_vec
 * @return T
 */
template <typename Builder, typename T> T convert_from_bn254_frs(Builder& builder, std::span<const fr<Builder>> fr_vec)
{
    if constexpr (IsAnyOf<T, fr<Builder>>) {
        ASSERT(fr_vec.size() == 1);
        return fr_vec[0];
    } else if constexpr (IsAnyOf<T, fq<Builder>>) {
        ASSERT(fr_vec.size() == 2);
        fq<Builder> result(fr_vec[0], fr_vec[1], 0, 0);
        return result;
    } else if constexpr (IsAnyOf<T, bn254_element<Builder>>) {
        using BaseField = fq<Builder>;
        constexpr size_t BASE_FIELD_SCALAR_SIZE = calc_num_bn254_frs<Builder, BaseField>();
        ASSERT(fr_vec.size() == 2 * BASE_FIELD_SCALAR_SIZE);
        bn254_element<Builder> result;
        result.x = convert_from_bn254_frs<Builder, BaseField>(builder, fr_vec.subspan(0, BASE_FIELD_SCALAR_SIZE));
        result.y = convert_from_bn254_frs<Builder, BaseField>(
            builder, fr_vec.subspan(BASE_FIELD_SCALAR_SIZE, BASE_FIELD_SCALAR_SIZE));
        return result;
    } else if constexpr (IsAnyOf<T, grumpkin_element<Builder>>) {
        using BaseField = fr<Builder>;
        constexpr size_t BASE_FIELD_SCALAR_SIZE = calc_num_bn254_frs<Builder, BaseField>();
        ASSERT(fr_vec.size() == 2 * BASE_FIELD_SCALAR_SIZE);
        grumpkin_element<Builder> result(
            convert_from_bn254_frs<Builder, fr<Builder>>(builder, fr_vec.subspan(0, BASE_FIELD_SCALAR_SIZE)),
            convert_from_bn254_frs<Builder, fr<Builder>>(
                builder, fr_vec.subspan(BASE_FIELD_SCALAR_SIZE, BASE_FIELD_SCALAR_SIZE)),
            false);
        return result;
    } else {
        // Array or Univariate
        T val;
        constexpr size_t FieldScalarSize = calc_num_bn254_frs<Builder, typename T::value_type>();
        ASSERT(fr_vec.size() == FieldScalarSize * std::tuple_size<T>::value);
        size_t i = 0;
        for (auto& x : val) {
            x = convert_from_bn254_frs<Builder, typename T::value_type>(
                builder, fr_vec.subspan(FieldScalarSize * i, FieldScalarSize));
            ++i;
        }
        return val;
    }
}

/**
 * @brief Conversion from transcript values to fr<Builder>s
 * @details We want to support the following types: bool, size_t, uint32_t, uint64_t, fr<Builder>, fq<Builder>,
 * bn254_element<Builder>, grumpkin_element<Builder,, bb::Univariate<FF, N>, std::array<FF,
 * N>, for FF = fr<Builder>/fq<Builder>, and N is arbitrary.
 * @tparam Builder
 * @tparam T
 * @param val
 * @return std::vector<fr<Builder>>
 */
template <typename Builder, typename T> std::vector<fr<Builder>> convert_to_bn254_frs(const T& val)
{
    if constexpr (IsAnyOf<T, fr<Builder>>) {
        std::vector<fr<Builder>> fr_vec{ val };
        return fr_vec;
    } else if constexpr (IsAnyOf<T, fq<Builder>>) {
        return convert_grumpkin_fr_to_bn254_frs(val);
    } else if constexpr (IsAnyOf<T, bn254_element<Builder>>) {
        using BaseField = fq<Builder>;
        auto fr_vec_x = convert_to_bn254_frs<Builder, BaseField>(val.x);
        auto fr_vec_y = convert_to_bn254_frs<Builder, BaseField>(val.y);
        std::vector<fr<Builder>> fr_vec(fr_vec_x.begin(), fr_vec_x.end());
        fr_vec.insert(fr_vec.end(), fr_vec_y.begin(), fr_vec_y.end());
        return fr_vec;
    } else if constexpr (IsAnyOf<T, grumpkin_element<Builder>>) {
        using BaseField = fr<Builder>;
        auto fr_vec_x = convert_to_bn254_frs<Builder, BaseField>(val.x);
        auto fr_vec_y = convert_to_bn254_frs<Builder, BaseField>(val.y);
        std::vector<fr<Builder>> fr_vec(fr_vec_x.begin(), fr_vec_x.end());
        fr_vec.insert(fr_vec.end(), fr_vec_y.begin(), fr_vec_y.end());
        return fr_vec;
    } else {
        // Array or Univariate
        std::vector<fr<Builder>> fr_vec;
        for (auto& x : val) {
            auto tmp_vec = convert_to_bn254_frs<Builder, typename T::value_type>(x);
            fr_vec.insert(fr_vec.end(), tmp_vec.begin(), tmp_vec.end());
        }
        return fr_vec;
    }
}

} // namespace bb::stdlib::field_conversion