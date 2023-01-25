#pragma once
#include <cstdint>
#include <ecc/curves/bn254/fr.hpp>

namespace bonk {
struct add_triple {
    uint32_t a;
    uint32_t b;
    uint32_t c;
    barretenberg::fr a_scaling;
    barretenberg::fr b_scaling;
    barretenberg::fr c_scaling;
    barretenberg::fr const_scaling;
};

struct add_quad {
    uint32_t a;
    uint32_t b;
    uint32_t c;
    uint32_t d;
    barretenberg::fr a_scaling;
    barretenberg::fr b_scaling;
    barretenberg::fr c_scaling;
    barretenberg::fr d_scaling;
    barretenberg::fr const_scaling;
};

struct mul_quad {
    uint32_t a;
    uint32_t b;
    uint32_t c;
    uint32_t d;
    barretenberg::fr mul_scaling;
    barretenberg::fr a_scaling;
    barretenberg::fr b_scaling;
    barretenberg::fr c_scaling;
    barretenberg::fr d_scaling;
    barretenberg::fr const_scaling;
};

struct mul_triple {
    uint32_t a;
    uint32_t b;
    uint32_t c;
    barretenberg::fr mul_scaling;
    barretenberg::fr c_scaling;
    barretenberg::fr const_scaling;
};

struct poly_triple {
    uint32_t a;
    uint32_t b;
    uint32_t c;
    barretenberg::fr q_m;
    barretenberg::fr q_l;
    barretenberg::fr q_r;
    barretenberg::fr q_o;
    barretenberg::fr q_c;
};

struct fixed_group_add_quad {
    uint32_t a;
    uint32_t b;
    uint32_t c;
    uint32_t d;
    barretenberg::fr q_x_1;
    barretenberg::fr q_x_2;
    barretenberg::fr q_y_1;
    barretenberg::fr q_y_2;
};

struct fixed_group_init_quad {
    barretenberg::fr q_x_1;
    barretenberg::fr q_x_2;
    barretenberg::fr q_y_1;
    barretenberg::fr q_y_2;
};

struct accumulator_triple {
    std::vector<uint32_t> left;
    std::vector<uint32_t> right;
    std::vector<uint32_t> out;
};

struct ecc_add_gate {
    uint32_t x1;
    uint32_t y1;
    uint32_t x2;
    uint32_t y2;
    uint32_t x3;
    uint32_t y3;
    barretenberg::fr endomorphism_coefficient;
    barretenberg::fr sign_coefficient;
};
} // namespace bonk