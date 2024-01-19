#pragma once
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/group/cycle_group.hpp"

#include "../../primitives/circuit_builders/circuit_builders.hpp"

namespace bb::plonk::stdlib {

using namespace bb;
/**
 * @brief stdlib class that evaluates in-circuit pedersen hashes, consistent with behavior in
 * crypto::pedersen_hash
 *
 * @tparam Builder
 */
template <typename Builder> class pedersen_hash {

  private:
    using field_ct = stdlib::field_t<Builder>;
    using bool_t = stdlib::bool_t<Builder>;
    using EmbeddedCurve = typename cycle_group<Builder>::Curve;
    using GeneratorContext = crypto::GeneratorContext<EmbeddedCurve>;
    using cycle_group = stdlib::cycle_group<Builder>;

  public:
    static field_ct hash(const std::vector<field_ct>& in, GeneratorContext context = {});
    // TODO health warnings!
    static field_ct hash_skip_field_validation(const std::vector<field_ct>& in, GeneratorContext context = {});
    static field_ct hash_buffer(const stdlib::byte_array<Builder>& input, GeneratorContext context = {});
};

} // namespace bb::plonk::stdlib
