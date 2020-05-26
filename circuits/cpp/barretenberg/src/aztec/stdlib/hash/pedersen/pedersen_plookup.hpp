// #pragma once
// #include <stdlib/types/turbo.hpp>

// namespace plonk {
// namespace stdlib {
// namespace pedersen {

// using namespace plonk::stdlib::types::turbo;

// field_ct compress_eight(std::array<field_ct, 8>& inputs, bool handle_edge_cases = false);

// // TODO: use unique generators for each range
// field_ct compress(std::vector<field_ct>& inputs, bool handle_edge_cases = false);

// field_ct compress(const field_ct& left,
//                   const field_ct& right,
//                   const size_t hash_index = 0,
//                   bool handle_edge_cases = false);

// byte_array_ct compress(const byte_array_ct& inputs);

// point_ct compress_to_point(const field_ct& left, const field_ct& right, const size_t hash_index = 0);

// } // namespace pedersen
// } // namespace stdlib
// } // namespace plonk

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

    static point hash_single(const field_t& in, const bool parity);

  public:
    static field_t compress(const field_t& left, const field_t& right);
};

extern template class pedersen_plookup<waffle::PLookupComposer>;
} // namespace stdlib
} // namespace plonk