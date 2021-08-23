#pragma once

#include <plonk/proof_system/constants.hpp>
#include <ecc/curves/bn254/fq.hpp>
#include <ecc/curves/bn254/fr.hpp>
#include <numeric/uint256/uint256.hpp>
#include <numeric/uintx/uintx.hpp>

#include "../byte_array/byte_array.hpp"
#include "../field/field.hpp"

#include "../composers/composers_fwd.hpp"

namespace plonk {
namespace stdlib {

template <typename Composer, typename T> class bigfield {
  public:
    struct Basis {
        uint512_t modulus;
        size_t num_bits;
    };

    struct Limb {
        Limb() {}
        Limb(const field_t<Composer>& input, const uint256_t max = uint256_t(0))
            : element(input)
        {
            if (input.witness_index == IS_CONSTANT) {
                maximum_value = uint256_t(input.additive_constant) + 1;
            } else if (max != uint256_t(0)) {
                maximum_value = max;
            } else {
                maximum_value = DEFAULT_MAXIMUM_LIMB;
            }
        }
        Limb(const Limb& other) = default;
        Limb(Limb&& other) = default;
        Limb& operator=(const Limb& other) = default;
        Limb& operator=(Limb&& other) = default;

        field_t<Composer> element;
        uint256_t maximum_value;
    };

    bigfield(const field_t<Composer>& low_bits, const field_t<Composer>& high_bits, const bool can_overflow = false);
    bigfield(Composer* parent_context = nullptr);
    bigfield(Composer* parent_context, const uint256_t& value);
    bigfield(const field_t<Composer>& a,
             const field_t<Composer>& b,
             const field_t<Composer>& c,
             const field_t<Composer>& d,
             const bool can_overflow = false)
        : bigfield((a + b * shift_1), (c + d * shift_1), can_overflow)
    {
        const auto limb_range_checks = [](const field_t<Composer>& limb, const bool overflow) {
            if (limb.is_constant()) {
                limb.create_range_constraint(overflow ? NUM_LIMB_BITS : NUM_LAST_LIMB_BITS);
            }
        };
        limb_range_checks(a, true);
        limb_range_checks(b, true);
        limb_range_checks(c, true);
        limb_range_checks(d, can_overflow);
    }
    bigfield(const byte_array<Composer>& bytes);
    bigfield(const bigfield& other);
    bigfield(bigfield&& other);

    static bigfield from_witness(Composer* ctx, const barretenberg::field<T>& input)
    {
        uint256_t input_u256(input);
        field_t<Composer> low(witness_t<Composer>(ctx, barretenberg::fr(input_u256.slice(0, NUM_LIMB_BITS * 2))));
        field_t<Composer> hi(
            witness_t<Composer>(ctx, barretenberg::fr(input_u256.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 4))));
        return bigfield(low, hi);
    }

    bigfield& operator=(const bigfield& other);
    bigfield& operator=(bigfield&& other);
    // code assumes modulus is at most 256 bits so good to define it via a uint256_t
    static constexpr uint256_t modulus = (uint256_t(T::modulus_0, T::modulus_1, T::modulus_2, T::modulus_3));
    static constexpr uint512_t modulus_u512 = uint512_t(modulus);
    static constexpr uint64_t NUM_LIMB_BITS = waffle::NUM_LIMB_BITS_IN_FIELD_SIMULATION;
    static constexpr uint64_t NUM_LAST_LIMB_BITS = modulus_u512.get_msb() + 1 - (NUM_LIMB_BITS * 3);
    static constexpr uint256_t DEFAULT_MAXIMUM_LIMB = (uint256_t(1) << NUM_LIMB_BITS) - uint256_t(1);
    static constexpr uint256_t DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB =
        (uint256_t(1) << NUM_LAST_LIMB_BITS) - uint256_t(1);
    static constexpr uint64_t LOG2_BINARY_MODULUS = NUM_LIMB_BITS * 4;
    static constexpr bool is_composite = true; // false only when fr is native

    static constexpr uint256_t prime_basis_maximum_limb =
        uint256_t(modulus_u512.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4));
    static constexpr Basis prime_basis{ uint512_t(barretenberg::fr::modulus), barretenberg::fr::modulus.get_msb() + 1 };
    static constexpr Basis binary_basis{ uint512_t(1) << LOG2_BINARY_MODULUS, LOG2_BINARY_MODULUS };
    static constexpr Basis target_basis{ modulus_u512, modulus_u512.get_msb() + 1 };
    static constexpr barretenberg::fr shift_1 = barretenberg::fr(uint256_t(1) << NUM_LIMB_BITS);
    static constexpr barretenberg::fr shift_2 = barretenberg::fr(uint256_t(1) << (NUM_LIMB_BITS * 2));
    static constexpr barretenberg::fr shift_3 = barretenberg::fr(uint256_t(1) << (NUM_LIMB_BITS * 3));
    static constexpr barretenberg::fr shift_right_1 = barretenberg::fr(1) / shift_1;
    static constexpr barretenberg::fr shift_right_2 = barretenberg::fr(1) / shift_2;
    static constexpr barretenberg::fr negative_prime_modulus_mod_binary_basis =
        -barretenberg::fr(uint256_t(modulus_u512));
    static constexpr uint512_t negative_prime_modulus = binary_basis.modulus - target_basis.modulus;
    static constexpr uint256_t neg_modulus_limbs_u256[4]{
        uint256_t(negative_prime_modulus.slice(0, NUM_LIMB_BITS).lo),
        uint256_t(negative_prime_modulus.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2).lo),
        uint256_t(negative_prime_modulus.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3).lo),
        uint256_t(negative_prime_modulus.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4).lo),
    };
    static constexpr barretenberg::fr neg_modulus_limbs[4]{
        barretenberg::fr(negative_prime_modulus.slice(0, NUM_LIMB_BITS).lo),
        barretenberg::fr(negative_prime_modulus.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2).lo),
        barretenberg::fr(negative_prime_modulus.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3).lo),
        barretenberg::fr(negative_prime_modulus.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4).lo),
    };

    byte_array<Composer> to_byte_array() const
    {
        byte_array<Composer> result(get_context());
        field_t<Composer> lo = binary_basis_limbs[0].element + (binary_basis_limbs[1].element * shift_1);
        field_t<Composer> hi = binary_basis_limbs[2].element + (binary_basis_limbs[3].element * shift_1);
        // n.b. this only works if NUM_LIMB_BITS * 2 is divisible by 8
        ASSERT((NUM_LIMB_BITS / 8) * 8 == NUM_LIMB_BITS);
        result.write(byte_array<Composer>(hi, 32 - (NUM_LIMB_BITS / 4)));
        result.write(byte_array<Composer>(lo, (NUM_LIMB_BITS / 4)));
        return result;
    }

    uint512_t get_value() const;
    uint512_t get_maximum_value() const;

    bigfield operator+(const bigfield& other) const;
    bigfield operator-(const bigfield& other) const;
    bigfield operator*(const bigfield& other) const;
    bigfield operator/(const bigfield& other) const;
    bigfield operator-() const { return bigfield(get_context(), uint256_t(0)) - *this; }

    bigfield operator+=(const bigfield& other)
    {
        *this = operator+(other);
        return *this;
    }
    bigfield operator-=(const bigfield& other)
    {
        *this = operator-(other);
        return *this;
    }
    bigfield operator*=(const bigfield& other)
    {
        *this = operator*(other);
        return *this;
    }
    bigfield operator/=(const bigfield& other)
    {
        *this = operator/(other);
        return *this;
    }

    bigfield sqr() const;
    bigfield sqradd(const std::vector<bigfield>& to_add) const;
    bigfield madd(const bigfield& to_mul, const std::vector<bigfield>& to_add) const;
    static bigfield div(const std::vector<bigfield>& numerators, const bigfield& denominator);

    bigfield conditional_negate(const bool_t<Composer>& predicate) const;
    bigfield conditional_select(const bigfield& other, const bool_t<Composer>& predicate) const;

    void assert_is_in_field() const;
    void assert_equal(const bigfield& other) const;
    void assert_is_not_equal(const bigfield& other) const;

    void self_reduce() const;

    bool is_constant() const { return prime_basis_limb.witness_index == IS_CONSTANT; }

    static bigfield one()
    {
        bigfield result(nullptr, uint256_t(1));
        return result;
    }
    static bigfield zero()
    {
        bigfield result(nullptr, uint256_t(0));
        return result;
    }

    Composer* get_context() const { return context; }

    static constexpr uint512_t get_maximum_unreduced_value()
    {
        uint1024_t maximum_product = uint1024_t(binary_basis.modulus) * uint1024_t(prime_basis.modulus);
        // TODO: compute square root (the following is a lower bound, so good for the CRT use)
        uint64_t maximum_product_bits = maximum_product.get_msb() - 1;
        return (uint512_t(1) << (maximum_product_bits >> 1)) - uint512_t(1);
    }
    // a (currently generous) upper bound on the log of number of fr additions in any of the class operations
    static constexpr uint64_t MAX_ADDITION_LOG = 10;
    // the rationale of the expression is we should not overflow Fr when applying any bigfield operation (e.g. *) and
    // starting with this max limb size
    static constexpr uint64_t MAX_UNREDUCED_LIMB_SIZE =
        (barretenberg::fr::modulus.get_msb() + 1) / 2 - MAX_ADDITION_LOG;

    static constexpr uint256_t get_maximum_unreduced_limb_value() { return uint256_t(1) << MAX_UNREDUCED_LIMB_SIZE; }

    Composer* context;
    mutable Limb binary_basis_limbs[4];
    mutable field_t<Composer> prime_basis_limb;

  private:
    static void evaluate_multiply_add(const bigfield& left,
                                      const bigfield& right_mul,
                                      const std::vector<bigfield>& to_add,
                                      const bigfield& quotient,
                                      const std::vector<bigfield>& remainders);
    static void verify_mod(const bigfield& left, const bigfield& quotient, const bigfield& remainder);

    static void evaluate_square_add(const bigfield& left,
                                    const std::vector<bigfield>& to_add,
                                    const bigfield& quotient,
                                    const bigfield& remainder);

    static void evaluate_product(const bigfield& left,
                                 const bigfield& right,
                                 const bigfield& quotient,
                                 const bigfield& remainder);
    void reduction_check() const;
}; // namespace stdlib

template <typename C, typename T> inline std::ostream& operator<<(std::ostream& os, bigfield<T, C> const& v)
{
    return os << v.get_value();
}

} // namespace stdlib
} // namespace plonk

#include "bigfield_impl.hpp"