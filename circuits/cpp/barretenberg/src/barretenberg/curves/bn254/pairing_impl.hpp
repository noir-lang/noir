#pragma once

#include "./fq12.hpp"
#include "./g1.hpp"
#include "./g2.hpp"

namespace barretenberg {
namespace pairing {
namespace {
constexpr fq two_inv = fq(2).invert();
inline constexpr g2::element mul_by_q(const g2::element& a)
{

    fq2 T0 = a.x.frobenius_map();
    fq2 T1 = a.y.frobenius_map();
    return {
        fq2::twist_mul_by_q_x() * T0,
        fq2::twist_mul_by_q_y() * T1,
        a.z.frobenius_map(),
    };
}
} // namespace
constexpr void doubling_step_for_flipped_miller_loop(g2::element& current, fq12::ell_coeffs& ell)
{
    fq2 a = current.x.mul_by_fq(two_inv);
    a *= current.y;

    fq2 b = current.y.sqr();
    fq2 c = current.z.sqr();
    fq2 d = c + c;
    d += c;
    fq2 e = d * fq2::twist_coeff_b();
    fq2 f = e + e;
    f += e;

    fq2 g = b + f;
    g = g.mul_by_fq(two_inv);
    fq2 h = current.y + current.z;
    h = h.sqr();
    fq2 i = b + c;
    h -= i;
    i = e - b;
    fq2 j = current.x.sqr();
    fq2 ee = e.sqr();
    fq2 k = b - f;
    current.x = a * k;

    k = ee + ee;
    k += ee;

    c = g.sqr();
    current.y = c - k;
    current.z = b * h;

    ell.o = fq6::mul_by_non_residue(i);

    ell.vw = -h;
    ell.vv = j + j;
    ell.vv += j;
}

constexpr void mixed_addition_step_for_flipped_miller_loop(const g2::element& base,
                                                           g2::element& Q,
                                                           fq12::ell_coeffs& line)
{
    fq2 d = base.x * Q.z;
    d = Q.x - d;

    fq2 e = base.y * Q.z;
    e = Q.y - e;

    fq2 f = d.sqr();
    fq2 g = e.sqr();
    fq2 h = d * f;
    fq2 i = Q.x * f;

    fq2 j = Q.z * g;
    j += h;
    j -= i;
    j -= i;

    Q.x = d * j;
    i -= j;
    i *= e;
    j = Q.y * h;
    Q.y = i - j;
    Q.z *= h;

    h = e * base.x;
    i = d * base.y;

    h -= i;
    line.o = fq6::mul_by_non_residue(h);

    line.vv = -e;
    line.vw = d;
}

constexpr void precompute_miller_lines(const g2::element& Q, miller_lines& lines)
{
    g2::element Q_neg{ Q.x, -Q.y, fq2::one() };
    g2::element work_point = Q;

    size_t it = 0;
    for (size_t i = 0; i < loop_length; ++i) {
        doubling_step_for_flipped_miller_loop(work_point, lines.lines[it]);
        ++it;
        if (loop_bits[i] == 1) {
            mixed_addition_step_for_flipped_miller_loop(Q, work_point, lines.lines[it]);
            ++it;
        } else if (loop_bits[i] == 3) {
            mixed_addition_step_for_flipped_miller_loop(Q_neg, work_point, lines.lines[it]);
            ++it;
        }
    }

    g2::element Q1 = mul_by_q(Q);
    g2::element Q2 = mul_by_q(Q1);
    Q2 = -Q2;
    mixed_addition_step_for_flipped_miller_loop(Q1, work_point, lines.lines[it]);
    ++it;
    mixed_addition_step_for_flipped_miller_loop(Q2, work_point, lines.lines[it]);
}

constexpr fq12 miller_loop(g1::element& P, miller_lines& lines)
{
    fq12 work_scalar = fq12::one();

    size_t it = 0;
    fq12::ell_coeffs work_line;

    for (size_t i = 0; i < loop_length; ++i) {
        work_scalar = work_scalar.sqr();

        work_line.o = lines.lines[it].o;
        work_line.vw = lines.lines[it].vw.mul_by_fq(P.y);
        work_line.vv = lines.lines[it].vv.mul_by_fq(P.x);
        work_scalar.self_sparse_mul(work_line);
        ++it;

        if (loop_bits[i] != 0) {
            work_line.o = lines.lines[it].o;
            work_line.vw = lines.lines[it].vw.mul_by_fq(P.y);
            work_line.vv = lines.lines[it].vv.mul_by_fq(P.x);
            work_scalar.self_sparse_mul(work_line);
            ++it;
        }
    }

    work_line.o = lines.lines[it].o;
    work_line.vw = lines.lines[it].vw.mul_by_fq(P.y);
    work_line.vv = lines.lines[it].vv.mul_by_fq(P.x);
    work_scalar.self_sparse_mul(work_line);
    ++it;
    work_line.o = lines.lines[it].o;
    work_line.vw = lines.lines[it].vw.mul_by_fq(P.y);
    work_line.vv = lines.lines[it].vv.mul_by_fq(P.x);
    work_scalar.self_sparse_mul(work_line);
    ++it;
    return work_scalar;
}

constexpr fq12 miller_loop_batch(const g1::element* points, const miller_lines* lines, size_t num_pairs)
{
    fq12 work_scalar = fq12::one();

    size_t it = 0;
    fq12::ell_coeffs work_line;

    for (size_t i = 0; i < loop_length; ++i) {
        work_scalar = work_scalar.sqr();
        for (size_t j = 0; j < num_pairs; ++j) {
            work_line.o = lines[j].lines[it].o;
            work_line.vw = lines[j].lines[it].vw.mul_by_fq(points[j].y);
            work_line.vv = lines[j].lines[it].vv.mul_by_fq(points[j].x);
            work_scalar.self_sparse_mul(work_line);
        }
        ++it;
        if (loop_bits[i] != 0) {
            for (size_t j = 0; j < num_pairs; ++j) {
                work_line.o = lines[j].lines[it].o;
                work_line.vw = lines[j].lines[it].vw.mul_by_fq(points[j].y);
                work_line.vv = lines[j].lines[it].vv.mul_by_fq(points[j].x);
                work_scalar.self_sparse_mul(work_line);
            }
            ++it;
        }
    }

    for (size_t j = 0; j < num_pairs; ++j) {
        work_line.o = lines[j].lines[it].o;
        work_line.vw = lines[j].lines[it].vw.mul_by_fq(points[j].y);
        work_line.vv = lines[j].lines[it].vv.mul_by_fq(points[j].x);
        work_scalar.self_sparse_mul(work_line);
    }
    ++it;
    for (size_t j = 0; j < num_pairs; ++j) {
        work_line.o = lines[j].lines[it].o;
        work_line.vw = lines[j].lines[it].vw.mul_by_fq(points[j].y);
        work_line.vv = lines[j].lines[it].vv.mul_by_fq(points[j].x);
        work_scalar.self_sparse_mul(work_line);
    }
    ++it;
    return work_scalar;
}

constexpr fq12 final_exponentiation_easy_part(const fq12& elt)
{
    fq12 a{ elt.c0, -elt.c1 };
    a *= elt.invert();
    return a * a.frobenius_map_two();
}

constexpr fq12 final_exponentiation_exp_by_neg_z(const fq12& elt)
{
    fq12 scalar{ elt };
    fq12 r = elt;

    for (size_t i = 0; i < neg_z_loop_length; ++i) {
        r = r.cyclotomic_squared();
        if (neg_z_loop_bits[i]) {
            r *= scalar;
        }
    }
    return r.unitary_inverse();
}

constexpr fq12 final_exponentiation_tricky_part(const fq12& elt)
{
    fq12 A = final_exponentiation_exp_by_neg_z(elt);
    fq12 B = A.cyclotomic_squared();
    fq12 C = B.cyclotomic_squared();
    fq12 D = B * C;
    fq12 E = final_exponentiation_exp_by_neg_z(D);
    fq12 F = E.cyclotomic_squared();
    fq12 G = final_exponentiation_exp_by_neg_z(F);
    fq12 H = D.unitary_inverse();
    fq12 I = G.unitary_inverse();
    fq12 J = I * E;
    fq12 K = J * H;
    fq12 L = B * K;
    fq12 M = E * K;
    fq12 N = M * elt;
    fq12 O = L.frobenius_map_one();
    fq12 P = O * N;
    fq12 Q = K.frobenius_map_two();
    fq12 R = P * Q;
    fq12 S = elt.unitary_inverse();
    fq12 T = L * S;
    fq12 U = T.frobenius_map_three();

    return R * U;
}

constexpr fq12 reduced_ate_pairing(const g1::affine_element& P_affine, const g2::affine_element& Q_affine)
{
    g1::element P(P_affine);
    g2::element Q(Q_affine);

    miller_lines lines;
    precompute_miller_lines(Q, lines);

    fq12 result = miller_loop(P, lines);
    result = final_exponentiation_easy_part(result);
    result = final_exponentiation_tricky_part(result);
    return result;
}

fq12 reduced_ate_pairing_batch_precomputed(const g1::affine_element* P_affines,
                                                    const miller_lines* lines,
                                                    const size_t num_points)
{
    g1::element* P = new g1::element[num_points];
    for (size_t i = 0; i < num_points; ++i) {
        P[i] = g1::element(P_affines[i]);
    }
    fq12 result = miller_loop_batch(&P[0], &lines[0], num_points);
    result = final_exponentiation_easy_part(result);
    result = final_exponentiation_tricky_part(result);
    delete[] P;
    return result;
}

fq12 reduced_ate_pairing_batch(const g1::affine_element* P_affines,
                                        const g2::affine_element* Q_affines,
                                        const size_t num_points)
{
    g1::element* P = new g1::element[num_points];
    g2::element* Q = new g2::element[num_points];
    miller_lines* lines = new miller_lines[num_points];
    for (size_t i = 0; i < num_points; ++i) {
        P[i] = g1::element(P_affines[i]);
        Q[i] = g2::element(Q_affines[i]);

        precompute_miller_lines(Q[i], lines[i]);
    }

    fq12 result = miller_loop_batch(&P[0], &lines[0], num_points);
    result = final_exponentiation_easy_part(result);
    result = final_exponentiation_tricky_part(result);
    delete[] P;
    delete[] Q;
    delete[] lines;
    return result;
}

} // namespace pairing
} // namespace barretenberg
