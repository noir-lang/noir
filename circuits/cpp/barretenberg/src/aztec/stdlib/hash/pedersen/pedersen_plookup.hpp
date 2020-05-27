#pragma once
#include "../../primitives/composers/composers_fwd.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/byte_array/byte_array.hpp"

namespace plonk {
namespace stdlib {

template <typename ComposerContext> class pedersen_plookup {
  private:
    typedef plonk::stdlib::field_t<ComposerContext> field_t;
    typedef plonk::stdlib::point<ComposerContext> point;
    typedef plonk::stdlib::byte_array<ComposerContext> byte_array;
    typedef plonk::stdlib::bool_t<ComposerContext> bool_t;

    enum AddType {
        LAMBDA,
        ONE,
        ONE_PLUS_LAMBDA,
    };

    static point hash_single(const field_t& in, const bool parity);
    static point add_points(const point& p1, const point& p2, const AddType add_type = ONE);
    static point compress_to_point(const field_t& left, const field_t& right);

  public:
    static field_t compress(const field_t& left, const field_t& right);
    static field_t compress(const std::vector<field_t>& inputs);

    static point encrypt(const std::vector<field_t>& inputs);
};

extern template class pedersen_plookup<waffle::PLookupComposer>;
} // namespace stdlib
} // namespace plonk