#pragma once
#include "barretenberg/polynomials/univariate.hpp"
#include <tuple>

namespace bb {

/**
 * @brief Generic templates for constructing a container of containers of varying length, where the various lengths are
 * specified in an array.
 *
 * @details Credit: https://stackoverflow.com/a/60440611
 */
template <template <typename, size_t, size_t> typename InnerContainer,
          typename ValueType,
          auto domain_end,
          size_t domain_start = 0,
          typename IS = decltype(std::make_index_sequence<domain_end.size()>())>
struct TupleOfContainersOverArray;
template <template <typename, size_t, size_t> typename InnerContainer,
          typename ValueType,
          auto domain_end,
          size_t domain_start,
          std::size_t... I>
struct TupleOfContainersOverArray<InnerContainer, ValueType, domain_end, domain_start, std::index_sequence<I...>> {
    using type = std::tuple<InnerContainer<ValueType, domain_end[I], domain_start>...>;
};

// Helpers
template <typename ValueType, size_t, size_t> using ExtractValueType = ValueType;

template <typename Tuple>
using HomogeneousTupleToArray = std::array<std::tuple_element_t<0, Tuple>, std::tuple_size_v<Tuple>>;

// Types needed for sumcheck and folding.
template <typename FF, auto LENGTHS>
using TupleOfUnivariates = typename TupleOfContainersOverArray<bb::Univariate, FF, LENGTHS, 0>::type;

template <typename FF, auto LENGTHS>
using TupleOfValues = typename TupleOfContainersOverArray<ExtractValueType, FF, LENGTHS>::type;

template <typename FF, auto LENGTHS> using ArrayOfValues = HomogeneousTupleToArray<TupleOfValues<FF, LENGTHS>>;

} // namespace bb