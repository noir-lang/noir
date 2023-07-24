#pragma once
#include "../bn254/fq.hpp"
#include "../bn254/fq12.hpp"
#include "../bn254/fq2.hpp"
#include "../bn254/fr.hpp"
#include "../bn254/g1.hpp"
#include "../bn254/g2.hpp"

namespace curve {
class BN254 {
  public:
    using ScalarField = barretenberg::fr;
    using BaseField = barretenberg::fq;
    using Group = typename barretenberg::g1;
    using Element = typename Group::element;
    using AffineElement = typename Group::affine_element;
    using G2AffineElement = typename barretenberg::g2::affine_element;
    using G2BaseField = typename barretenberg::fq2;
    using TargetField = barretenberg::fq12;
};
} // namespace curve