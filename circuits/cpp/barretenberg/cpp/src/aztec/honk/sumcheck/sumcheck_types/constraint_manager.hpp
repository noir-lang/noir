#pragma once
#include <cstddef>
#include <array>
#include <tuple>
#include "barycentric_data.hpp"
#include "univariate.hpp"

namespace honk::sumcheck {

template <typename... Constraints> class ConstraintManager {
    // template <class Fr, size_t MAX_CONSTRAINT_LENGTH, typename... Constraints> class ConstraintManager {
  public:
    static constexpr size_t NUM_CONSTRAINTS = sizeof...(Constraints);
    // TODO(cody): const correctness
    std::tuple<Constraints...> constraints;
    std::tuple<typename Constraints::UnivariateClass...> univariate_accumulators;
    // TODO(cody): make barycentric stuff static and put in here, rather than constraints?
    //             First need to figure out how max length (determined by flavour) is supplied.
    // static constexpr auto barycentric_data =
    //     std::tuple(BarycentricData<Fr, Constraints::CONSTRAINT_LENGTH, MAX_CONSTRAINT_LENGTH>()...)
    ConstraintManager()
        : constraints(Constraints()...)
        , univariate_accumulators(typename Constraints::UnivariateClass()...){};
};
} // namespace honk::sumcheck