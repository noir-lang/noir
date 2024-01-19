// TODO(@zac-wiliamson #2341 delete this file once we migrate to new hash standard

#pragma once
#include "../generators/generator_data.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include <array>

namespace bb::crypto {

/**
 * @brief Performs pedersen commitments!
 *
 * To commit to a size-n list of field elements `x`, a commitment is defined as:
 *
 *      Commit(x) = x[0].g[0] + x[1].g[1] + ... + x[n-1].g[n-1]
 *
 * Where `g` is a list of generator points defined by `generator_data`
 *
 */
template <typename Curve> class pedersen_commitment_base {
  public:
    using AffineElement = typename Curve::AffineElement;
    using Element = typename Curve::Element;
    using Fr = typename Curve::ScalarField;
    using Fq = typename Curve::BaseField;
    using Group = typename Curve::Group;
    using GeneratorContext = typename crypto::GeneratorContext<Curve>;

    static AffineElement commit_native(const std::vector<Fq>& inputs, GeneratorContext context = {});
};

using pedersen_commitment = pedersen_commitment_base<curve::Grumpkin>;
} // namespace bb::crypto
