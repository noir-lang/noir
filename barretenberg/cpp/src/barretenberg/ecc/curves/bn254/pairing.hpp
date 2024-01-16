#pragma once

#include <random>

#include "./fq12.hpp"
#include "./fq2.hpp"
#include "./fq6.hpp"
#include "./g1.hpp"
#include "./g2.hpp"

namespace bb::pairing {
constexpr size_t loop_length = 64;
constexpr size_t neg_z_loop_length = 62;
constexpr size_t precomputed_coefficients_length = 87;

constexpr std::array<uint8_t, loop_length> loop_bits{ 1, 0, 1, 0, 0, 0, 3, 0, 3, 0, 0, 0, 3, 0, 1, 0, 3, 0, 0, 3, 0, 0,
                                                      0, 0, 0, 1, 0, 0, 3, 0, 1, 0, 0, 3, 0, 0, 0, 0, 3, 0, 1, 0, 0, 0,
                                                      3, 0, 3, 0, 0, 1, 0, 0, 0, 3, 0, 0, 3, 0, 1, 0, 1, 0, 0, 0 };

constexpr std::array<bool, neg_z_loop_length> neg_z_loop_bits{
    false, false, false, true,  false, false, true,  true,  true, false, true,  false, false, true,  true,  false,
    false, true,  false, false, true,  false, true,  false, true, true,  false, true,  false, false, false, true,
    false, false, true,  false, true,  false, false, true,  true, false, true,  false, false, true,  false, false,
    false, false, true,  false, false, true,  true,  true,  true, true,  false, false, false, true
};

struct miller_lines {
    std::array<fq12::ell_coeffs, precomputed_coefficients_length> lines;
};

constexpr void doubling_step_for_flipped_miller_loop(g2::element& current, fq12::ell_coeffs& ell);

constexpr void mixed_addition_step_for_flipped_miller_loop(const g2::element& base,
                                                           g2::element& Q,
                                                           fq12::ell_coeffs& line);

constexpr void precompute_miller_lines(const g2::element& Q, miller_lines& lines);

constexpr fq12 miller_loop(const g1::element& P, const miller_lines& lines);

constexpr fq12 miller_loop_batch(const g1::element* points, const miller_lines* lines, size_t num_pairs);

constexpr void final_exponentiation_easy_part(const fq12& elt, fq12& r);

constexpr void final_exponentiation_exp_by_neg_z(const fq12& elt, fq12& r);

constexpr void final_exponentiation_tricky_part(const fq12& elt, fq12& r);

constexpr fq12 reduced_ate_pairing(const g1::affine_element& P_affine, const g2::affine_element& Q_affine);

inline fq12 reduced_ate_pairing_batch(const g1::affine_element* P_affines,
                                      const g2::affine_element* Q_affines,
                                      size_t num_points);

inline fq12 reduced_ate_pairing_batch_precomputed(const g1::affine_element* P_affines,
                                                  const miller_lines* lines,
                                                  size_t num_points);

} // namespace bb::pairing

#include "./pairing_impl.hpp"