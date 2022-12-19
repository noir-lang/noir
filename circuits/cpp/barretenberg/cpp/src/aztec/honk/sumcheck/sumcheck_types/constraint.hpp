#pragma once
#include <cstddef>
#include <array>
#include <tuple>
#include "./multivariates.hpp"
// #include "honk/sumcheck/sumcheck_types/univariate.hpp"

namespace honk::sumcheck {

template <typename Fr> class Constraint {
  public:
    static const size_t HONK_CONSTRAINT_LENGTH = 5; // TODO(luke): move to something more global
    static constexpr size_t NUM_HONK_MULTIVARIATES = MULTIVARIATE::COUNT;
};

} // namespace honk::sumcheck
