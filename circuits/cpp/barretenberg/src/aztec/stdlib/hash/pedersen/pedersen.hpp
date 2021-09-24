#pragma once
#include <crypto/pedersen/pedersen.hpp>
#include "../../primitives/composers/composers_fwd.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/point/point.hpp"
#include "../../primitives/byte_array/byte_array.hpp"

namespace plonk {
namespace stdlib {

template <typename ComposerContext> class pedersen {
  private:
    typedef plonk::stdlib::field_t<ComposerContext> field_t;
    typedef plonk::stdlib::point<ComposerContext> point;
    typedef plonk::stdlib::byte_array<ComposerContext> byte_array;
    typedef plonk::stdlib::bool_t<ComposerContext> bool_t;

    static point hash_single(const field_t& in,
                             const crypto::pedersen::generator_index_t hash_index,
                             const bool validate_edge_cases = false,
                             const bool validate_input_is_in_field = true);
    static point accumulate(const std::vector<point>& to_accumulate);
    static point conditionally_accumulate(const std::vector<point>& to_accumulate, const std::vector<field_t>& inputs);

  public:
    static field_t compress(const field_t& left,
                            const field_t& right,
                            const size_t hash_index = 0,
                            const bool handle_edge_cases = false,
                            const bool validate_input_is_in_field = true);

    static field_t compress(const std::vector<field_t>& inputs,
                            const bool handle_edge_cases = false,
                            const size_t hash_index = 0);

    template <size_t T>
    static field_t compress(const std::array<field_t, T>& inputs, const bool handle_edge_cases = true)
    {
        std::vector<field_t> in(inputs.begin(), inputs.end());
        return compress(in, handle_edge_cases);
    }
    static field_t compress(const byte_array& inputs);

    static point compress_to_point(const field_t& left, const field_t& right, const size_t hash_index = 0);

    static point commit(const std::vector<field_t>& inputs,
                        const size_t hash_index = 0,
                        const bool handle_edge_cases = true);

    static void validate_wnaf_is_in_field(ComposerContext* ctx,
                                          const std::vector<uint32_t>& accumulator,
                                          const field_t& in,
                                          const bool validate_edge_cases);
};

extern template class pedersen<waffle::StandardComposer>;
extern template class pedersen<waffle::TurboComposer>;
extern template class pedersen<waffle::PlookupComposer>;

} // namespace stdlib
} // namespace plonk