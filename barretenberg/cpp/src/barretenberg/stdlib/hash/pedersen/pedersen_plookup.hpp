#pragma once
#include "../../primitives/circuit_builders/circuit_builders_fwd.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/packed_byte_array/packed_byte_array.hpp"
#include "../../primitives/point/point.hpp"

namespace proof_system::plonk {
namespace stdlib {

template <typename Builder> class pedersen_plookup_hash {
  private:
    typedef stdlib::field_t<Builder> field_t;
    typedef stdlib::point<Builder> point;
    typedef stdlib::packed_byte_array<Builder> packed_byte_array;
    typedef stdlib::bool_t<Builder> bool_t;

    enum AddType {
        LAMBDA,
        ONE,
        ONE_PLUS_LAMBDA,
    };

  public:
    static point add_points(const point& p1, const point& p2, const AddType add_type = ONE);

    static point hash_single(const field_t& in, const bool parity, const bool skip_range_check = false);

    static field_t hash_multiple(const std::vector<field_t>& in, const size_t hash_index = 0);
};

EXTERN_STDLIB_ULTRA_TYPE(pedersen_plookup_hash);
} // namespace stdlib
} // namespace proof_system::plonk