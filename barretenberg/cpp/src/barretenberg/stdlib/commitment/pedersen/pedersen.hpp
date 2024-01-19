#pragma once
#include "../../primitives/byte_array/byte_array.hpp"
#include "../../primitives/field/field.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/stdlib/primitives/group/cycle_group.hpp"

namespace bb::plonk::stdlib {

template <typename CircuitBuilder> class pedersen_commitment {
  private:
    using bool_t = stdlib::bool_t<CircuitBuilder>;
    using field_t = stdlib::field_t<CircuitBuilder>;
    using EmbeddedCurve = typename cycle_group<CircuitBuilder>::Curve;
    using GeneratorContext = crypto::GeneratorContext<EmbeddedCurve>;
    using cycle_group = stdlib::cycle_group<CircuitBuilder>;
    using cycle_scalar = typename stdlib::cycle_group<CircuitBuilder>::cycle_scalar;

  public:
    static cycle_group commit(const std::vector<field_t>& inputs, GeneratorContext context = {});
    static cycle_group commit(const std::vector<std::pair<field_t, GeneratorContext>>& input_pairs);
};

} // namespace bb::plonk::stdlib