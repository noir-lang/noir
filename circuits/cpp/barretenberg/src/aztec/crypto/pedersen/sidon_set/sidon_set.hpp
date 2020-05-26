#pragma once

#include <array>
#include <vector>

#include <numeric/uintx/uintx.hpp>
#include <ecc/fields/field.hpp>
#include <ecc/fields/field2.hpp>

namespace crypto {

namespace pedersen {
namespace sidon {
// find the smallest prime p ,where p >= lower_bound and (p mod 4 = 3)
// the latter condition is because our fq2 implementation requires -1 to be a quadratic non residue
constexpr uint64_t compute_nearest_safe_prime(const uint64_t lower_bound)
{
    uint64_t iterator = lower_bound;
    while (true) {
        if ((iterator & 0x3UL) == 0x3UL) {
            bool prime = true;
            for (uint64_t i = 2; i * i <= iterator; ++i) {
                if (iterator % i == 0) {
                    prime = false;
                    break;
                }
            }
            if (prime) {
                return iterator;
            }
        }
        ++iterator;
    }
    return 0;
}

inline std::vector<uint64_t> compute_prime_factors(uint64_t input)
{
    uint64_t target = input;
    std::vector<uint64_t> factors;
    while (target % 2 == 0) {
        factors.push_back(2);
        target >>= 1;
    }

    for (uint64_t i = 3; i * i <= target; i += 2) {
        while (target % i == 0) {
            factors.push_back(i);
            target /= i;
        }
    }

    if (target > 2) {
        factors.push_back(target);
    }

    std::sort(factors.begin(), factors.end());

    std::vector<uint64_t> unique_factors;

    unique_factors.push_back(factors[0]);
    for (size_t i = 1; i < factors.size(); ++i) {
        if (factors[i] != factors[i - 1]) {
            unique_factors.push_back(factors[i]);
        }
    }
    return unique_factors;
}

template <uint64_t sidon_prime> constexpr std::array<uint64_t, 4> get_r_squared()
{
    uint1024_t r_squared = uint1024_t(1) << 256;
    uint1024_t prime = uint1024_t(uint512_t(uint256_t(sidon_prime)));
    r_squared = r_squared * r_squared;
    r_squared = r_squared % prime;
    uint256_t out = r_squared.lo.lo;
    return { out.data[0], out.data[1], out.data[2], out.data[3] };
}

template <uint64_t sidon_prime> constexpr uint64_t get_r_inv()
{
    uint256_t prime(sidon_prime);
    uint512_t r{ 0, 1 };
    uint512_t q{ -prime, 0 };
    uint256_t q_inv = q.invmod(r).lo;
    return q_inv.data[0];
}

template <uint64_t sidon_prime> class SidonFqParams {
  public:
    static constexpr uint64_t modulus_0 = sidon_prime;
    static constexpr uint64_t modulus_1 = 0;
    static constexpr uint64_t modulus_2 = 0;
    static constexpr uint64_t modulus_3 = 0;

    static constexpr std::array<uint64_t, 4> r_squared = get_r_squared<sidon_prime>();
    static constexpr uint64_t r_squared_0 = r_squared[0];
    static constexpr uint64_t r_squared_1 = r_squared[1];
    static constexpr uint64_t r_squared_2 = r_squared[2];
    static constexpr uint64_t r_squared_3 = r_squared[3];

    static constexpr uint64_t cube_root_0 = 0;
    static constexpr uint64_t cube_root_1 = 0;
    static constexpr uint64_t cube_root_2 = 0;
    static constexpr uint64_t cube_root_3 = 0;

    static constexpr uint64_t primitive_root_0 = 0UL;
    static constexpr uint64_t primitive_root_1 = 0UL;
    static constexpr uint64_t primitive_root_2 = 0UL;
    static constexpr uint64_t primitive_root_3 = 0UL;

    static constexpr uint64_t endo_g1_lo = 0;
    static constexpr uint64_t endo_g1_mid = 0;
    static constexpr uint64_t endo_g1_hi = 0;
    static constexpr uint64_t endo_g2_lo = 0;
    static constexpr uint64_t endo_g2_mid = 0;
    static constexpr uint64_t endo_minus_b1_lo = 0;
    static constexpr uint64_t endo_minus_b1_mid = 0;
    static constexpr uint64_t endo_b2_lo = 0;
    static constexpr uint64_t endo_b2_mid = 0;

    static constexpr uint64_t r_inv = get_r_inv<sidon_prime>();

    static constexpr uint64_t coset_generators_0[8]{ 0, 0, 0, 0, 0, 0, 0, 0 };

    static constexpr uint64_t coset_generators_1[8]{ 0, 0, 0, 0, 0, 0, 0, 0 };

    static constexpr uint64_t coset_generators_2[8]{ 0, 0, 0, 0, 0, 0, 0, 0 };

    static constexpr uint64_t coset_generators_3[8]{ 0, 0, 0, 0, 0, 0, 0, 0 };
};

template <typename Fq> struct SidonFq2Params {
    static constexpr Fq twist_coeff_b_0{ 0, 0, 0, 0 };
    static constexpr Fq twist_coeff_b_1{ 0, 0, 0, 0 };
    static constexpr Fq twist_mul_by_q_x_0{ 0, 0, 0, 0 };
    static constexpr Fq twist_mul_by_q_x_1{ 0, 0, 0, 0 };
    static constexpr Fq twist_mul_by_q_y_0{ 0, 0, 0, 0 };
    static constexpr Fq twist_mul_by_q_y_1{ 0, 0, 0, 0 };
    static constexpr Fq twist_cube_root_0{ 0, 0, 0, 0 };
    static constexpr Fq twist_cube_root_1{ 0, 0, 0, 0 };
};

template <uint64_t set_size> using sidon_fq = barretenberg::field<SidonFqParams<compute_nearest_safe_prime(set_size)>>;

template <uint64_t set_size>
using sidon_fq2 = barretenberg::field2<sidon_fq<set_size>, SidonFq2Params<sidon_fq<set_size>>>;

template <uint64_t set_size> inline sidon_fq2<set_size> get_sidon_generator()
{
    typedef sidon_fq2<set_size> fq2;
    constexpr uint64_t q = fq2::modulus.data[0];

    std::vector<uint64_t> fq2_prime_factors = compute_prime_factors(q * q - 1);

    std::vector<uint64_t> primitive_exponents;
    for (const auto factor : fq2_prime_factors) {
        primitive_exponents.push_back((q * q - 1) / factor);
    }

    const auto is_generator = [&primitive_exponents](uint64_t x, uint64_t y) {
        fq2 target = fq2(x, y);
        bool is_primitive = true;
        for (const auto exponent : primitive_exponents) {
            fq2 powered = target.pow(exponent);
            if (powered == fq2(1)) {
                is_primitive = false;
                break;
            }
        }
        return is_primitive;
    };

    uint64_t a = 1;
    uint64_t b = 1;
    while (!is_generator(a, b)) {
        a = (a + 1) % q;
        if (a == 0) {
            b = (b + 1) % q;
        }
    }
    return { a, b };
}

/**
 * Computes a Sidon set of a defined size. Implements section 3.5.2 of the following paper:
 * Bshouty, Nader. "Testers and their applications." Proceedings of the 5th conference on Innovations in theoretical
 *computer science. 2014.
 *
 * For our pedersen hash, we desire a set of integers, where the sums of any two set members are unique.
 *
 * 1. find a prime `q` that is >= our target set size
 * 2. find a generator `g` of the field Fq2
 * 3. find the set of integers `a`, where `g^a - g` produces an element of Fq <- this is our Sidon set
 *
 * The technique can be extended to find Sidon sequences that are greater than 2 (e.g. sets where all combinations of 3
 *set members are unique) This can be done by using a higher degree field extension - currently not implemented
 **/
template <uint64_t set_size> inline std::vector<uint64_t> compute_sidon_set()
{
    constexpr uint64_t q = compute_nearest_safe_prime(set_size);

    const auto generator = get_sidon_generator<set_size>();

    // 1: get generator of prime field subgroup
    // 2: compute g^i - g for all i in maximum
    std::vector<uint64_t> set_members;

    auto accumulator = generator;
    for (size_t a = 1; a < q * q - 1; ++a) {
        const auto target_element = accumulator - generator;
        if (target_element.c1 == 0) {
            set_members.push_back(a);
        }
        if (set_members.size() == set_size) {
            break;
        }
        accumulator *= generator;
    }

    std::sort(set_members.begin(), set_members.end());
    return set_members;
}
} // namespace sidon
} // namespace pedersen
} // namespace crypto