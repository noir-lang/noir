#pragma once
#include <array>
#include <plonk/composer/plookup_tables/plookup_tables.hpp>
#include <stdlib/primitives/uint/uint.hpp>
#include <plonk/composer/composer_base.hpp>

#include <numeric/bitop/sparse_form.hpp>

#include "../../primitives/field/field.hpp"
#include "../../primitives/packed_byte_array/packed_byte_array.hpp"

namespace waffle {
class PlookupComposer;
} // namespace waffle

namespace plonk {
namespace stdlib {
namespace sha256_plookup {

struct sparse_ch_value {
    field_t<waffle::PlookupComposer> normal;
    field_t<waffle::PlookupComposer> sparse;
    field_t<waffle::PlookupComposer> rot6;
    field_t<waffle::PlookupComposer> rot11;
    field_t<waffle::PlookupComposer> rot25;
};

struct sparse_maj_value {
    field_t<waffle::PlookupComposer> normal;
    field_t<waffle::PlookupComposer> sparse;
    field_t<waffle::PlookupComposer> rot2;
    field_t<waffle::PlookupComposer> rot13;
    field_t<waffle::PlookupComposer> rot22;
};

struct sparse_witness_limbs {
    sparse_witness_limbs(const field_t<waffle::PlookupComposer>& in = 0)
    {
        normal = in;
        has_sparse_limbs = false;
    }
    sparse_witness_limbs(const sparse_witness_limbs& other) = default;
    sparse_witness_limbs(sparse_witness_limbs&& other) = default;

    sparse_witness_limbs& operator=(const sparse_witness_limbs& other) = default;
    sparse_witness_limbs& operator=(sparse_witness_limbs&& other) = default;

    field_t<waffle::PlookupComposer> normal;

    std::array<field_t<waffle::PlookupComposer>, 4> sparse_limbs;

    std::array<field_t<waffle::PlookupComposer>, 4> rotated_limbs;

    bool has_sparse_limbs = false;
};

struct sparse_value {
    sparse_value(const field_t<waffle::PlookupComposer>& in = 0)
    {
        normal = in;
        if (normal.witness_index == UINT32_MAX) {
            sparse = field_t<waffle::PlookupComposer>(
                in.get_context(),
                barretenberg::fr(numeric::map_into_sparse_form<16>(uint256_t(in.get_value()).data[0])));
        }
    }

    sparse_value(const sparse_value& other) = default;
    sparse_value(sparse_value&& other) = default;

    sparse_value& operator=(const sparse_value& other) = default;
    sparse_value& operator=(sparse_value&& other) = default;

    field_t<waffle::PlookupComposer> normal;
    field_t<waffle::PlookupComposer> sparse;
};

sparse_witness_limbs convert_witness(const field_t<waffle::PlookupComposer>& w);

std::array<field_t<waffle::PlookupComposer>, 64> extend_witness(
    const std::array<field_t<waffle::PlookupComposer>, 16>& w_in);

field_t<waffle::PlookupComposer> choose(sparse_value& e, const sparse_value& f, const sparse_value& g);
field_t<waffle::PlookupComposer> majority(sparse_value& a, const sparse_value& b, const sparse_value& c);

std::array<field_t<waffle::PlookupComposer>, 8> sha256_block(
    const std::array<field_t<waffle::PlookupComposer>, 8>& h_init,
    const std::array<field_t<waffle::PlookupComposer>, 16>& input);

packed_byte_array<waffle::PlookupComposer> sha256(const packed_byte_array<waffle::PlookupComposer>& input);
} // namespace sha256_plookup
} // namespace stdlib
} // namespace plonk
