#pragma once

#include <numeric/uintx/uintx.hpp>
#include "secp256k1.hpp"

namespace secp256k1_params {
struct basis_vectors {
    uint64_t endo_g1_lo = 0;
    uint64_t endo_g1_mid = 0;
    uint64_t endo_g1_hi = 0;
    uint64_t endo_g1_hihi = 0;
    uint64_t endo_g2_lo = 0;
    uint64_t endo_g2_mid = 0;
    uint64_t endo_g2_hi = 0;
    uint64_t endo_g2_hihi = 0;
    uint64_t endo_minus_b1_lo = 0;
    uint64_t endo_minus_b1_mid = 0;
    uint64_t endo_b2_lo = 0;
    uint64_t endo_b2_mid = 0;
    uint64_t endo_a1_lo = 0;
    uint64_t endo_a1_mid = 0;
    uint64_t endo_a1_hi = 0;
    uint64_t endo_a2_lo = 0;
    uint64_t endo_a2_mid = 0;
    uint64_t endo_a2_hi = 0;

    bool real = false;
};
static basis_vectors get_endomorphism_basis_vectors(const secp256k1::fr& lambda)
{
    uint512_t approximate_square_root;
    uint512_t z = (uint512_t(secp256k1::fr::modulus) + uint512_t(2)) >> 1;
    uint512_t y = uint512_t(secp256k1::fr::modulus);
    while (z < y) {
        y = z;
        z = (uint512_t(secp256k1::fr::modulus) / z + z) >> 1;
    }
    approximate_square_root = y;
    // Run the extended greatest common divisor algorithm until out * (\lambda + 1) < approximate_square_root

    uint512_t u(lambda);
    uint512_t v(secp256k1::fr::modulus);
    uint512_t x1 = 1;
    uint512_t y1 = 0;
    uint512_t x2 = 0;
    uint512_t y2 = 1;

    uint512_t a0 = 0;
    uint512_t b0 = 0;

    uint512_t a1 = 0;
    uint512_t b1 = 0;

    uint512_t a2 = 0;
    uint512_t b2 = 0;

    uint512_t prevOut = 0;
    uint512_t i = 0;
    uint512_t out = 0;
    uint512_t x = 0;

    while (u != 0) {
        uint512_t q = v / u;
        out = v - uint512_t(uint512_t(q) * uint512_t(u));
        x = x2 - (q * x1);
        uint512_t y = y2 - (q * y1);
        if ((a1 == 0) && (out < approximate_square_root)) {
            a0 = -prevOut;
            b0 = x1;
            a1 = -out;
            b1 = x;
        } else if ((a1 > 0) && (++i == 2)) {
            break;
        }
        prevOut = out;

        v = u;
        u = out;
        x2 = x1;
        x1 = x;
        y2 = y1;
        y1 = y;
    }

    a2 = -out;
    b2 = x;

    uint512_t len1 = (a1 * a1) + (b1 * b1);
    uint512_t len2 = (a2 * a2) + (b2 * b2);
    if (len2 >= len1) {
        a2 = a0;
        b2 = b0;
    }

    if (a1.get_msb() >= 128) {
        a1 = -a1;
        b1 = -b1;
    }
    if (a2.get_msb() >= 128) {
        a2 = -a2;
        b2 = -b2;
    }

    uint512_t minus_b1 = -b1;
    uint512_t shift256 = uint512_t(1) << 384;
    uint512_t g1 = (-b1 * shift256) / uint512_t(secp256k1::fr::modulus);
    uint512_t g2 = (b2 * shift256) / uint512_t(secp256k1::fr::modulus);

    basis_vectors result;
    result.endo_g1_lo = g1.lo.data[0];
    result.endo_g1_mid = g1.lo.data[1];
    result.endo_g1_hi = g1.lo.data[2];
    result.endo_g1_hihi = g1.lo.data[3];
    result.endo_g2_lo = g2.lo.data[0];
    result.endo_g2_mid = g2.lo.data[1];
    result.endo_g2_hi = g2.lo.data[2];
    result.endo_g2_hihi = g2.lo.data[3];
    result.endo_minus_b1_lo = minus_b1.lo.data[0];
    result.endo_minus_b1_mid = minus_b1.lo.data[1];
    result.endo_b2_lo = b2.lo.data[0];
    result.endo_b2_mid = b2.lo.data[1];
    result.endo_a1_lo = a1.lo.data[0];
    result.endo_a1_mid = a1.lo.data[1];
    result.endo_a1_hi = a1.lo.data[2];
    result.endo_a2_lo = a2.lo.data[0];
    result.endo_a2_mid = a2.lo.data[1];
    result.endo_a2_hi = a2.lo.data[2];
    return result;
}

static std::pair<secp256k1::fq, secp256k1::fr> get_endomorphism_scalars()
{
    // find beta \in secp256k1::fq and lambda \in secp256k1::fr such that:

    // 1. beta^3 = 1 mod q
    // 2. lambda^3 = 1 mod r
    // 3. for [P] \in G with coordinates (P.x, P.y) \in secp256k1::fq:
    //      \lambda.[P] = (\beta . P.x, P.y)
    const secp256k1::fq beta = secp256k1::fq::cube_root_of_unity();
    const secp256k1::fr lambda = secp256k1::fr::cube_root_of_unity();

    if (beta * beta * beta != secp256k1::fq(1)) {
        std::cerr << "beta is not a cube root of unity" << std::endl;
    }
    if (lambda * lambda * lambda != secp256k1::fr(1)) {
        std::cerr << "lambda is not a cube root of unity" << std::endl;
    }

    secp256k1::g1::element P = secp256k1::g1::one;
    secp256k1::g1::element endoP = P;
    endoP.x *= beta;

    if (P * lambda == endoP) {
        return { beta, lambda };
    }
    endoP.x *= beta;
    if ((P * lambda) == endoP) {
        return { beta * beta, lambda };
    }
    endoP.y = -endoP.y;
    std::cerr << "could not find endomorphism scalars???" << std::endl;
    return { secp256k1::fq(0), secp256k1::fr(0) };
}
}; // namespace secp256k1_params