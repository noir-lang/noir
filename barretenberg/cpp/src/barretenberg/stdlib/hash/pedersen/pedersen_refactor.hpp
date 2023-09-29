#pragma once
#include "../../primitives/field/field.hpp"
#include "../../primitives/point/point.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/stdlib/primitives/group/cycle_group.hpp"

#include "../../primitives/circuit_builders/circuit_builders.hpp"

namespace proof_system::plonk::stdlib {

using namespace barretenberg;
/**
 * @brief stdlib class that evaluates in-circuit pedersen hashes, consistent with behavior in
 * crypto::pedersen_hash_refactor
 *
 * @tparam ComposerContext
 */
template <typename ComposerContext> class pedersen_hash_refactor {

  private:
    using field_t = stdlib::field_t<ComposerContext>;
    using point = stdlib::point<ComposerContext>;
    using bool_t = stdlib::bool_t<ComposerContext>;
    using EmbeddedCurve = typename cycle_group<ComposerContext>::Curve;
    using generator_data = crypto::generator_data<EmbeddedCurve>;

  public:
    // TODO(@suyash67) as part of refactor project, can we remove this and replace with `hash`
    // (i.e. simplify the name as we no longer have a need for `hash_single`)
    static field_t hash_multiple(const std::vector<field_t>& in,
                                 size_t hash_index = 0,
                                 const generator_data* generator_context = generator_data::get_default_generators(),
                                 bool validate_inputs_in_field = true);

    static field_t hash(const std::vector<field_t>& in,
                        size_t hash_index = 0,
                        const generator_data* generator_context = generator_data::get_default_generators(),
                        bool validate_inputs_in_field = true);
};

EXTERN_STDLIB_TYPE(pedersen_hash_refactor);

} // namespace proof_system::plonk::stdlib
