#pragma once
#include "../../primitives/composers/composers_fwd.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/point/point.hpp"
#include "../../primitives/packed_byte_array/packed_byte_array.hpp"

namespace plonk {
namespace stdlib {

template <typename ComposerContext> class pedersen_plookup {
  private:
    typedef plonk::stdlib::field_t<ComposerContext> field_t;
    typedef plonk::stdlib::point<ComposerContext> point;
    typedef plonk::stdlib::packed_byte_array<ComposerContext> packed_byte_array;
    typedef plonk::stdlib::bool_t<ComposerContext> bool_t;

    enum AddType {
        LAMBDA,
        ONE,
        ONE_PLUS_LAMBDA,
    };

    static point hash_single(const field_t& in, const bool parity);
    static point add_points(const point& p1, const point& p2, const AddType add_type = ONE);

  public:
    static field_t compress(const field_t& left, const field_t& right);
    static field_t compress(const std::vector<field_t>& inputs, const size_t hash_index = 0);
    static field_t compress(const packed_byte_array& input) { return compress(input.get_limbs()); }

    template <size_t T> static field_t compress(const std::array<field_t, T>& inputs)
    {
        std::vector<field_t> in(inputs.begin(), inputs.end());
        return compress(in);
    }

    static point merkle_damgard_compress(const std::vector<field_t>& inputs, const field_t& iv);

    static point commit(const std::vector<field_t>& inputs, const size_t hash_index = 0);

    static point compress_to_point(const field_t& left, const field_t& right);
};

extern template class pedersen_plookup<waffle::UltraComposer>;
} // namespace stdlib
} // namespace plonk