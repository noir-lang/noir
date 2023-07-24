#pragma once
#include "barretenberg/proof_system/plookup_tables/plookup_tables.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include <array>

#include "barretenberg/numeric/bitop/sparse_form.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders_fwd.hpp"

#include "../../primitives/field/field.hpp"
#include "../../primitives/packed_byte_array/packed_byte_array.hpp"

namespace proof_system::plonk {
namespace stdlib {
namespace sha256_plookup {

template <typename Composer> struct sparse_ch_value {
    field_t<Composer> normal;
    field_t<Composer> sparse;
    field_t<Composer> rot6;
    field_t<Composer> rot11;
    field_t<Composer> rot25;
};

template <typename Composer> struct sparse_maj_value {
    field_t<Composer> normal;
    field_t<Composer> sparse;
    field_t<Composer> rot2;
    field_t<Composer> rot13;
    field_t<Composer> rot22;
};

template <typename Composer> struct sparse_witness_limbs {
    sparse_witness_limbs(const field_t<Composer>& in = 0)
    {
        normal = in;
        has_sparse_limbs = false;
    }
    sparse_witness_limbs(const sparse_witness_limbs& other) = default;
    sparse_witness_limbs(sparse_witness_limbs&& other) = default;

    sparse_witness_limbs& operator=(const sparse_witness_limbs& other) = default;
    sparse_witness_limbs& operator=(sparse_witness_limbs&& other) = default;

    field_t<Composer> normal;

    std::array<field_t<Composer>, 4> sparse_limbs;

    std::array<field_t<Composer>, 4> rotated_limbs;

    bool has_sparse_limbs = false;
};

template <typename Composer> struct sparse_value {
    sparse_value(const field_t<Composer>& in = 0)
    {
        normal = in;
        if (normal.witness_index == IS_CONSTANT) {
            sparse = field_t<Composer>(
                in.get_context(),
                barretenberg::fr(numeric::map_into_sparse_form<16>(uint256_t(in.get_value()).data[0])));
        }
    }

    sparse_value(const sparse_value& other) = default;
    sparse_value(sparse_value&& other) = default;

    sparse_value& operator=(const sparse_value& other) = default;
    sparse_value& operator=(sparse_value&& other) = default;

    field_t<Composer> normal;
    field_t<Composer> sparse;
};

template <typename Composer> sparse_witness_limbs<Composer> convert_witness(const field_t<Composer>& w);

template <typename Composer>
std::array<field_t<Composer>, 64> extend_witness(const std::array<field_t<Composer>, 16>& w_in);

template <typename Composer>
field_t<Composer> choose(sparse_value<Composer>& e, const sparse_value<Composer>& f, const sparse_value<Composer>& g);
template <typename Composer>
field_t<Composer> majority(sparse_value<Composer>& a, const sparse_value<Composer>& b, const sparse_value<Composer>& c);

template <typename Composer>
std::array<field_t<Composer>, 8> sha256_block(const std::array<field_t<Composer>, 8>& h_init,
                                              const std::array<field_t<Composer>, 16>& input);

template <typename Composer> packed_byte_array<Composer> sha256(const packed_byte_array<Composer>& input);
} // namespace sha256_plookup
} // namespace stdlib
} // namespace proof_system::plonk
