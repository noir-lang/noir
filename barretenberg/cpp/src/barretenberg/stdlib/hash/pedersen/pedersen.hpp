#pragma once
#include "../../primitives/circuit_builders/circuit_builders_fwd.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/point/point.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"

namespace proof_system::plonk {
namespace stdlib {

using namespace barretenberg;
template <typename ComposerContext> class pedersen_hash {

  private:
    typedef stdlib::field_t<ComposerContext> field_t;
    typedef stdlib::point<ComposerContext> point;
    typedef stdlib::bool_t<ComposerContext> bool_t;

  private:
    static point add_points(const point& first, const point& second);

    static point hash_single_internal(const field_t& in,
                                      const crypto::generators::generator_index_t hash_index,
                                      const bool validate_input_is_in_field = true);

  public:
    static void validate_wnaf_is_in_field(ComposerContext* ctx, const std::vector<uint32_t>& accumulator);

    static point accumulate(const std::vector<point>& to_accumulate);

    static point hash_single(const field_t& in,
                             const crypto::generators::generator_index_t hash_index,
                             const bool validate_input_is_in_field = true);

    static point commit_single(const field_t& in,
                               const crypto::generators::generator_index_t hash_index,
                               const bool validate_input_is_in_field = true);

    static field_t hash_multiple(const std::vector<field_t>& in,
                                 const size_t hash_index = 0,
                                 const bool validate_inputs_in_field = true);
};

EXTERN_STDLIB_TYPE(pedersen_hash);

} // namespace stdlib
} // namespace proof_system::plonk
