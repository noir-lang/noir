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

template <typename ComposerContext> class pedersen {
  private:
    typedef plonk::stdlib::field_t<ComposerContext> field_t;
    typedef plonk::stdlib::point<ComposerContext> point;
    typedef plonk::stdlib::byte_array<ComposerContext> byte_array;
    typedef plonk::stdlib::bool_t<ComposerContext> bool_t;

    static point hash_single(const field_t& in, const size_t hash_index, const bool validate_edge_cases = false);
    static field_t accumulate(std::vector<point>& to_accumulate);
    static field_t conditionally_accumulate(std::vector<point>& to_accumulate, std::vector<field_t>& inputs);

  public:
    static field_t compress_eight(std::array<field_t, 8>& inputs, bool handle_edge_cases = false);

    // TODO: use unique generators for each range
    static field_t compress(std::vector<field_t>& inputs, bool handle_edge_cases = false);

    static field_t compress(const field_t& left, const field_t& right, const size_t hash_index, bool handle_edge_cases);

    static field_t compress(const field_t& left, const field_t& right, const size_t hash_index)
    {
        return compress(left, right, hash_index, false);
    }

    static field_t compress(const field_t& left, const field_t& right) { return compress(left, right, 0, false); }

    static byte_array compress(const byte_array& inputs);

    static point compress_to_point(const field_t& left, const field_t& right, const size_t hash_index = 0);
};

extern template class pedersen<waffle::TurboComposer>;
extern template class pedersen<waffle::PLookupComposer>;
} // namespace stdlib
} // namespace plonk