#pragma once

#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/polynomials/univariate.hpp"

namespace bb::field_conversion {

/**
 * @brief Calculates number of bb::fr required to represent the input type
 * @details We want to support the following types: bool, size_t, uint32_t, uint64_t, bb::fr, grumpkin::fr,
 * curve::BN254::AffineElement, curve::Grumpkin::AffineElement, bb::Univariate<FF, N>, std::array<FF, N>, for
 * FF = bb::fr/grumpkin::fr, and N is arbitrary
 * @tparam T
 * @return constexpr size_t
 */
template <typename T> constexpr size_t calc_num_bn254_frs();

constexpr size_t calc_num_bn254_frs(bb::fr* /*unused*/)
{
    return 1;
}

constexpr size_t calc_num_bn254_frs(grumpkin::fr* /*unused*/)
{
    return 2;
}

template <std::integral T> constexpr size_t calc_num_bn254_frs(T* /*unused*/)
{
    return 1; // meant for integral types that are less than 254 bits
}

constexpr size_t calc_num_bn254_frs(curve::BN254::AffineElement* /*unused*/)
{
    return 2 * calc_num_bn254_frs<curve::BN254::BaseField>();
}

constexpr size_t calc_num_bn254_frs(curve::Grumpkin::AffineElement* /*unused*/)
{
    return 2 * calc_num_bn254_frs<curve::Grumpkin::BaseField>();
}

template <typename T, std::size_t N> constexpr size_t calc_num_bn254_frs(std::array<T, N>* /*unused*/)
{
    return N * calc_num_bn254_frs<T>();
}

template <typename T, std::size_t N> constexpr size_t calc_num_bn254_frs(bb::Univariate<T, N>* /*unused*/)
{
    return N * calc_num_bn254_frs<T>();
}

template <typename T> constexpr size_t calc_num_bn254_frs()
{
    return calc_num_bn254_frs(static_cast<T*>(nullptr));
}

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
template <typename T> T convert_from_bn254_frs(std::span<const bb::fr> fr_vec);

bool convert_from_bn254_frs(std::span<const bb::fr> fr_vec, bool* /*unused*/);

template <std::integral T> inline T convert_from_bn254_frs(std::span<const bb::fr> fr_vec, T* /*unused*/)
{
    ASSERT(fr_vec.size() == 1);
    return static_cast<T>(fr_vec[0]);
}

bb::fr convert_from_bn254_frs(std::span<const bb::fr> fr_vec, bb::fr* /*unused*/);

grumpkin::fr convert_from_bn254_frs(std::span<const bb::fr> fr_vec, grumpkin::fr* /*unused*/);

curve::BN254::AffineElement convert_from_bn254_frs(std::span<const bb::fr> fr_vec,
                                                   curve::BN254::AffineElement* /*unused*/);

curve::Grumpkin::AffineElement convert_from_bn254_frs(std::span<const bb::fr> fr_vec,
                                                      curve::Grumpkin::AffineElement* /*unused*/);

template <size_t N>
inline std::array<bb::fr, N> convert_from_bn254_frs(std::span<const bb::fr> fr_vec, std::array<bb::fr, N>* /*unused*/)
{
    std::array<bb::fr, N> val;
    for (size_t i = 0; i < N; ++i) {
        val[i] = fr_vec[i];
    }
    return val;
}

template <size_t N>
inline std::array<grumpkin::fr, N> convert_from_bn254_frs(std::span<const bb::fr> fr_vec,
                                                          std::array<grumpkin::fr, N>* /*unused*/)
{
    std::array<grumpkin::fr, N> val;
    for (size_t i = 0; i < N; ++i) {
        std::vector<bb::fr> fr_vec_tmp{ fr_vec[2 * i],
                                        fr_vec[2 * i + 1] }; // each pair of consecutive elements is a grumpkin::fr
        val[i] = convert_from_bn254_frs<grumpkin::fr>(fr_vec_tmp);
    }
    return val;
}

template <size_t N>
inline Univariate<bb::fr, N> convert_from_bn254_frs(std::span<const bb::fr> fr_vec, Univariate<bb::fr, N>* /*unused*/)
{
    Univariate<bb::fr, N> val;
    for (size_t i = 0; i < N; ++i) {
        val.evaluations[i] = fr_vec[i];
    }
    return val;
}

template <size_t N>
inline Univariate<grumpkin::fr, N> convert_from_bn254_frs(std::span<const bb::fr> fr_vec,
                                                          Univariate<grumpkin::fr, N>* /*unused*/)
{
    Univariate<grumpkin::fr, N> val;
    for (size_t i = 0; i < N; ++i) {
        std::vector<bb::fr> fr_vec_tmp{ fr_vec[2 * i], fr_vec[2 * i + 1] };
        val.evaluations[i] = convert_from_bn254_frs<grumpkin::fr>(fr_vec_tmp);
    }
    return val;
}

template <typename T> T convert_from_bn254_frs(std::span<const bb::fr> fr_vec)
{
    return convert_from_bn254_frs(fr_vec, static_cast<T*>(nullptr));
}

/**
 * @brief Conversion from transcript values to bb::frs
 * @details We want to support the following types: bool, size_t, uint32_t, uint64_t, bb::fr, grumpkin::fr,
 * curve::BN254::AffineElement, curve::Grumpkin::AffineElement, bb::Univariate<FF, N>, std::array<FF, N>, for
 * FF = bb::fr/grumpkin::fr, and N is arbitrary.
 * @tparam T
 * @param val
 * @return std::vector<bb::fr>
 */
template <std::integral T> std::vector<bb::fr> inline convert_to_bn254_frs(const T& val)
{
    std::vector<bb::fr> fr_vec{ val };
    return fr_vec;
}

std::vector<bb::fr> convert_to_bn254_frs(const grumpkin::fr& val);

std::vector<bb::fr> convert_to_bn254_frs(const bb::fr& val);

std::vector<bb::fr> convert_to_bn254_frs(const curve::BN254::AffineElement& val);

std::vector<bb::fr> convert_to_bn254_frs(const curve::Grumpkin::AffineElement& val);

template <size_t N> std::vector<bb::fr> inline convert_to_bn254_frs(const std::array<bb::fr, N>& val)
{
    std::vector<bb::fr> fr_vec(val.begin(), val.end());
    return fr_vec;
}

template <size_t N> std::vector<bb::fr> inline convert_to_bn254_frs(const std::array<grumpkin::fr, N>& val)
{
    std::vector<bb::fr> fr_vec;
    for (size_t i = 0; i < N; ++i) {
        auto tmp_vec = convert_to_bn254_frs(val[i]);
        fr_vec.insert(fr_vec.end(), tmp_vec.begin(), tmp_vec.end());
    }
    return fr_vec;
}

template <size_t N> std::vector<bb::fr> inline convert_to_bn254_frs(const bb::Univariate<bb::fr, N>& val)
{
    std::vector<bb::fr> fr_vec;
    for (size_t i = 0; i < N; ++i) {
        auto tmp_vec = convert_to_bn254_frs(val.evaluations[i]);
        fr_vec.insert(fr_vec.end(), tmp_vec.begin(), tmp_vec.end());
    }
    return fr_vec;
}

template <size_t N> std::vector<bb::fr> inline convert_to_bn254_frs(const bb::Univariate<grumpkin::fr, N>& val)
{
    std::vector<bb::fr> fr_vec;
    for (size_t i = 0; i < N; ++i) {
        auto tmp_vec = convert_to_bn254_frs(val.evaluations[i]);
        fr_vec.insert(fr_vec.end(), tmp_vec.begin(), tmp_vec.end());
    }
    return fr_vec;
}

template <typename AllValues> std::vector<bb::fr> inline convert_to_bn254_frs(const AllValues& val)
{
    auto data = val.get_all();
    std::vector<bb::fr> fr_vec;
    for (auto& item : data) {
        auto tmp_vec = convert_to_bn254_frs(item);
        fr_vec.insert(fr_vec.end(), tmp_vec.begin(), tmp_vec.end());
    }
    return fr_vec;
}

} // namespace bb::field_conversion
