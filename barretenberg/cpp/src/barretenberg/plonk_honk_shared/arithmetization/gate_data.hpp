#pragma once
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include <cstdint>

// TODO(#557): The field-specific aliases for gates should be removed and the type could be explicit when this
// structures are used to avoid having foo_gate and foo_gate_grumpkin (i.e. use foo_gate<field> instead). Moreover, we
// need to ensure the read/write functions handle grumpkin gates as well.
namespace bb {
template <typename FF> struct add_triple_ {
    uint32_t a;
    uint32_t b;
    uint32_t c;
    FF a_scaling;
    FF b_scaling;
    FF c_scaling;
    FF const_scaling;
};

template <typename FF> struct add_quad_ {
    uint32_t a;
    uint32_t b;
    uint32_t c;
    uint32_t d;
    FF a_scaling;
    FF b_scaling;
    FF c_scaling;
    FF d_scaling;
    FF const_scaling;
};
template <typename FF> struct mul_quad_ {
    uint32_t a;
    uint32_t b;
    uint32_t c;
    uint32_t d;
    FF mul_scaling;
    FF a_scaling;
    FF b_scaling;
    FF c_scaling;
    FF d_scaling;
    FF const_scaling;
};
template <typename FF> struct mul_triple_ {
    uint32_t a;
    uint32_t b;
    uint32_t c;
    FF mul_scaling;
    FF c_scaling;
    FF const_scaling;
};
template <typename FF> struct poly_triple_ {
    uint32_t a;
    uint32_t b;
    uint32_t c;
    FF q_m;
    FF q_l;
    FF q_r;
    FF q_o;
    FF q_c;

    friend bool operator==(poly_triple_<FF> const& lhs, poly_triple_<FF> const& rhs) = default;
};
using poly_triple = poly_triple_<bb::fr>;
struct ecc_op_tuple {
    uint32_t op;
    uint32_t x_lo;
    uint32_t x_hi;
    uint32_t y_lo;
    uint32_t y_hi;
    uint32_t z_1;
    uint32_t z_2;
    bool return_is_infinity;
};

template <typename B, typename FF> inline void read(B& buf, poly_triple_<FF>& constraint)
{
    using serialize::read;
    read(buf, constraint.a);
    read(buf, constraint.b);
    read(buf, constraint.c);
    read(buf, constraint.q_m);
    read(buf, constraint.q_l);
    read(buf, constraint.q_r);
    read(buf, constraint.q_o);
    read(buf, constraint.q_c);
}
template <typename B, typename FF> inline void write(B& buf, poly_triple_<FF> const& constraint)
{
    using serialize::write;
    write(buf, constraint.a);
    write(buf, constraint.b);
    write(buf, constraint.c);
    write(buf, constraint.q_m);
    write(buf, constraint.q_l);
    write(buf, constraint.q_r);
    write(buf, constraint.q_o);
    write(buf, constraint.q_c);
}

template <typename FF> struct fixed_group_add_quad_ {
    uint32_t a;
    uint32_t b;
    uint32_t c;
    uint32_t d;
    FF q_x_1;
    FF q_x_2;
    FF q_y_1;
    FF q_y_2;
};
template <typename FF> struct fixed_group_init_quad_ {
    FF q_x_1;
    FF q_x_2;
    FF q_y_1;
    FF q_y_2;
};
template <typename FF> struct accumulator_triple_ {
    std::vector<uint32_t> left;
    std::vector<uint32_t> right;
    std::vector<uint32_t> out;
};
template <typename FF> struct ecc_add_gate_ {
    uint32_t x1;
    uint32_t y1;
    uint32_t x2;
    uint32_t y2;
    uint32_t x3;
    uint32_t y3;
    FF sign_coefficient;
};
template <typename FF> struct ecc_dbl_gate_ {
    uint32_t x1;
    uint32_t y1;
    uint32_t x3;
    uint32_t y3;
};

template <typename FF> struct databus_lookup_gate_ {
    uint32_t index;
    uint32_t value;
};

/* External gate data for poseidon2 external round*/
template <typename FF> struct poseidon2_external_gate_ {
    uint32_t a;
    uint32_t b;
    uint32_t c;
    uint32_t d;
    size_t round_idx;
};

/* Internal gate data for poseidon2 internal round*/
template <typename FF> struct poseidon2_internal_gate_ {
    uint32_t a;
    uint32_t b;
    uint32_t c;
    uint32_t d;
    size_t round_idx;
};
} // namespace bb
