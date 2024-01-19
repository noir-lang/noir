#pragma once
#include "barretenberg/proof_system/plookup_tables/plookup_tables.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include <array>

#include "barretenberg/numeric/bitop/sparse_form.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders_fwd.hpp"

#include "../../primitives/field/field.hpp"
#include "../../primitives/packed_byte_array/packed_byte_array.hpp"

namespace bb::plonk {
namespace stdlib {
namespace sha256_plookup {

template <typename Builder> struct sparse_ch_value {
    field_t<Builder> normal;
    field_t<Builder> sparse;
    field_t<Builder> rot6;
    field_t<Builder> rot11;
    field_t<Builder> rot25;
};

template <typename Builder> struct sparse_maj_value {
    field_t<Builder> normal;
    field_t<Builder> sparse;
    field_t<Builder> rot2;
    field_t<Builder> rot13;
    field_t<Builder> rot22;
};

template <typename Builder> struct sparse_witness_limbs {
    sparse_witness_limbs(const field_t<Builder>& in = 0)
    {
        normal = in;
        has_sparse_limbs = false;
    }
    sparse_witness_limbs(const sparse_witness_limbs& other) = default;
    sparse_witness_limbs(sparse_witness_limbs&& other) = default;

    sparse_witness_limbs& operator=(const sparse_witness_limbs& other) = default;
    sparse_witness_limbs& operator=(sparse_witness_limbs&& other) = default;

    field_t<Builder> normal;

    std::array<field_t<Builder>, 4> sparse_limbs;

    std::array<field_t<Builder>, 4> rotated_limbs;

    bool has_sparse_limbs = false;
};

template <typename Builder> struct sparse_value {
    sparse_value(const field_t<Builder>& in = 0)
    {
        normal = in;
        if (normal.witness_index == IS_CONSTANT) {
            sparse = field_t<Builder>(in.get_context(),
                                      bb::fr(numeric::map_into_sparse_form<16>(uint256_t(in.get_value()).data[0])));
        }
    }

    sparse_value(const sparse_value& other) = default;
    sparse_value(sparse_value&& other) = default;

    sparse_value& operator=(const sparse_value& other) = default;
    sparse_value& operator=(sparse_value&& other) = default;

    field_t<Builder> normal;
    field_t<Builder> sparse;
};

template <typename Builder> sparse_witness_limbs<Builder> convert_witness(const field_t<Builder>& w);

template <typename Builder>
std::array<field_t<Builder>, 64> extend_witness(const std::array<field_t<Builder>, 16>& w_in);

template <typename Builder>
field_t<Builder> choose(sparse_value<Builder>& e, const sparse_value<Builder>& f, const sparse_value<Builder>& g);
template <typename Builder>
field_t<Builder> majority(sparse_value<Builder>& a, const sparse_value<Builder>& b, const sparse_value<Builder>& c);

template <typename Builder>
std::array<field_t<Builder>, 8> sha256_block(const std::array<field_t<Builder>, 8>& h_init,
                                             const std::array<field_t<Builder>, 16>& input);

template <typename Builder> packed_byte_array<Builder> sha256(const packed_byte_array<Builder>& input);
} // namespace sha256_plookup
} // namespace stdlib
} // namespace bb::plonk
