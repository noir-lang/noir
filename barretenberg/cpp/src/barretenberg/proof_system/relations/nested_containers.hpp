#pragma once
#include "barretenberg/polynomials/univariate.hpp"
#include <tuple>

namespace proof_system {

/**
 * @brief Generic templates for constructing a container of containers of varying length, where the various lengths are
 * specified in an array.
 *
 * @details Credit: https://stackoverflow.com/a/60440611
 */
template <template <typename, size_t> typename InnerContainer,
          typename ValueType,
          auto LENGTHS,
          typename IS = decltype(std::make_index_sequence<LENGTHS.size()>())>
struct TupleOfContainersOverArray;
template <template <typename, size_t> typename InnerContainer, typename ValueType, auto LENGTHS, std::size_t... I>
struct TupleOfContainersOverArray<InnerContainer, ValueType, LENGTHS, std::index_sequence<I...>> {
    using type = std::tuple<InnerContainer<ValueType, LENGTHS[I]>...>;
};

// Helpers
template <typename ValueType, size_t> using ExtractValueType = ValueType;

template <typename Tuple>
using HomogeneousTupleToArray = std::array<std::tuple_element_t<0, Tuple>, std::tuple_size_v<Tuple>>;

// Types needed for sumcheck and folding.
template <typename FF, auto LENGTHS>
using TupleOfUnivariates = typename TupleOfContainersOverArray<barretenberg::Univariate, FF, LENGTHS>::type;

template <typename FF, auto LENGTHS>
using TupleOfValues = typename TupleOfContainersOverArray<ExtractValueType, FF, LENGTHS>::type;

template <typename FF, auto LENGTHS> using ArrayOfValues = HomogeneousTupleToArray<TupleOfValues<FF, LENGTHS>>;

} // namespace proof_system