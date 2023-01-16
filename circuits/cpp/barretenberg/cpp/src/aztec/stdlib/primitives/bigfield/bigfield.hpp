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
    typedef T TParams;
    typedef barretenberg::field<T> native;

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
        friend std::ostream& operator<<(std::ostream& os, const Limb& a)
        {
            os << "{ " << a.element << " < " << a.maximum_value << " }";
            return os;
        }
        Limb(const Limb& other) = default;
        Limb(Limb&& other) = default;
        Limb& operator=(const Limb& other) = default;
        Limb& operator=(Limb&& other) = default;

        field_t<Composer> element;
        uint256_t maximum_value;
    };

    bigfield(const field_t<Composer>& low_bits,
             const field_t<Composer>& high_bits,
             const bool can_overflow = false,
             const size_t maximum_bitlength = 0);
    bigfield(Composer* parent_context = nullptr);
    bigfield(Composer* parent_context, const uint256_t& value);

    // we assume the limbs have already been normalized!
    bigfield(const field_t<Composer>& a,
             const field_t<Composer>& b,
             const field_t<Composer>& c,
             const field_t<Composer>& d,
             const bool can_overflow = false)
    {
        context = a.context;
        binary_basis_limbs[0] = Limb(field_t(a));
        binary_basis_limbs[1] = Limb(field_t(b));
        binary_basis_limbs[2] = Limb(field_t(c));
        binary_basis_limbs[3] =
            Limb(field_t(d), can_overflow ? DEFAULT_MAXIMUM_LIMB : DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);
        prime_basis_limb =
            (binary_basis_limbs[3].element * shift_3)
                .add_two(binary_basis_limbs[2].element * shift_2, binary_basis_limbs[1].element * shift_1);
        prime_basis_limb += (binary_basis_limbs[0].element);
    };

    // we assume the limbs have already been normalized!
    bigfield(const field_t<Composer>& a,
             const field_t<Composer>& b,
             const field_t<Composer>& c,
             const field_t<Composer>& d,
             const field_t<Composer>& prime_limb,
             const bool can_overflow = false)
    {
        context = a.context;
        binary_basis_limbs[0] = Limb(field_t(a));
        binary_basis_limbs[1] = Limb(field_t(b));
        binary_basis_limbs[2] = Limb(field_t(c));
        binary_basis_limbs[3] =
            Limb(field_t(d), can_overflow ? DEFAULT_MAXIMUM_LIMB : DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);
        prime_basis_limb = prime_limb;
    };

    bigfield(const byte_array<Composer>& bytes);
    bigfield(const bigfield& other);
    bigfield(bigfield&& other);

    static bigfield create_from_u512_as_witness(Composer* ctx,
                                                const uint512_t& value,
                                                const bool can_overflow = false,
                                                const size_t maximum_bitlength = 0);

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
    static constexpr uint64_t NUM_LIMB_BITS = 68;
    static constexpr uint64_t NUM_LAST_LIMB_BITS = modulus_u512.get_msb() + 1 - (NUM_LIMB_BITS * 3);
    static constexpr uint1024_t DEFAULT_MAXIMUM_REMAINDER =
        (uint1024_t(1) << (NUM_LIMB_BITS * 3 + NUM_LAST_LIMB_BITS)) - uint1024_t(1);
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

    bigfield add_to_lower_limb(const field_t<Composer>& other, uint256_t other_maximum_value) const;
    bigfield operator+(const bigfield& other) const;
    bigfield operator-(const bigfield& other) const;
    bigfield operator*(const bigfield& other) const;

    /**
     * FOR TESTING PURPOSES ONLY DO NOT USE THIS IN PRODUCTION CODE FOR THE LOVE OF GOD!
     **/
    bigfield bad_mul(const bigfield& other) const;

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

    static void perform_reductions_for_mult_madd(std::vector<bigfield>& mul_left,
                                                 std::vector<bigfield>& mul_right,
                                                 const std::vector<bigfield>& to_add);

    static bigfield mult_madd(const std::vector<bigfield>& mul_left,
                              const std::vector<bigfield>& mul_right,
                              const std::vector<bigfield>& to_add,
                              bool fix_remainder_to_zero = false);

    static bigfield dual_madd(const bigfield& left_a,
                              const bigfield& right_a,
                              const bigfield& left_b,
                              const bigfield& right_b,
                              const std::vector<bigfield>& to_add);

    // compute -(mul_left * mul_right + ...to_sub) / (divisor)
    // We can evaluate this relationship with only one set of quotient/remainder range checks
    static bigfield msub_div(const std::vector<bigfield>& mul_left,
                             const std::vector<bigfield>& mul_right,
                             const bigfield& divisor,
                             const std::vector<bigfield>& to_sub,
                             bool enable_divisor_nz_check = false);

    static bigfield sum(const std::vector<bigfield>& terms);
    static bigfield internal_div(const std::vector<bigfield>& numerators,
                                 const bigfield& denominator,
                                 bool check_for_zero);

    static bigfield div_without_denominator_check(const std::vector<bigfield>& numerators, const bigfield& denominator);
    static bigfield div_check_denominator_nonzero(const std::vector<bigfield>& numerators, const bigfield& denominator);

    bigfield conditional_negate(const bool_t<Composer>& predicate) const;
    bigfield conditional_select(const bigfield& other, const bool_t<Composer>& predicate) const;

    void assert_is_in_field() const;
    void assert_equal(const bigfield& other) const;
    void assert_is_not_equal(const bigfield& other) const;

    void self_reduce() const;

    bool is_constant() const { return prime_basis_limb.witness_index == IS_CONSTANT; }

    /**
     * Create a public one constant
     * */
    static bigfield one()
    {
        bigfield result(nullptr, uint256_t(1));
        return result;
    }

    /**
     * Create a public zero constant
     * */
    static bigfield zero()
    {
        bigfield result(nullptr, uint256_t(0));
        return result;
    }

    /**
     * @brief Create an unreduced 0 ~ p*k, where p*k is the minimal multiple of modulus that should be reduced
     *
     * @details We need it for division. If we always add this element during division, then we never run into the
     * formula-breaking situation
     */
    static constexpr bigfield unreduced_zero()
    {
        uint512_t multiple_of_modulus = ((get_maximum_unreduced_value() / modulus_u512) + 1) * modulus_u512;
        auto msb = multiple_of_modulus.get_msb();

        bigfield result(nullptr, uint256_t(0));
        result.binary_basis_limbs[0] = Limb(barretenberg::fr(multiple_of_modulus.slice(0, NUM_LIMB_BITS).lo));
        result.binary_basis_limbs[1] =
            Limb(barretenberg::fr(multiple_of_modulus.slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS).lo));
        result.binary_basis_limbs[2] =
            Limb(barretenberg::fr(multiple_of_modulus.slice(2 * NUM_LIMB_BITS, 3 * NUM_LIMB_BITS).lo));
        result.binary_basis_limbs[3] = Limb(barretenberg::fr(multiple_of_modulus.slice(3 * NUM_LIMB_BITS, msb + 1).lo));
        result.prime_basis_limb = field_t<Composer>((multiple_of_modulus % uint512_t(field_t<Composer>::modulus)).lo);
        return result;
    }

    /**
     * Create a witness form a constant. This way the value of the witness is fixed and public.
     **/
    void convert_constant_to_witness(Composer* composer)
    {
        context = composer;
        for (auto& limb : binary_basis_limbs) {
            limb.element.convert_constant_to_witness(context);
        }
        prime_basis_limb.convert_constant_to_witness(context);
    }

    /**
     * Fix a witness. The value of the witness is constrained with a selector
     **/
    void fix_witness()
    {
        for (auto& limb : binary_basis_limbs) {
            limb.element.fix_witness();
        }
        prime_basis_limb.fix_witness();
    }

    Composer* get_context() const { return context; }

    static constexpr uint512_t get_maximum_unreduced_value(const size_t num_products = 1)
    {
        // return (uint512_t(1) << 256);
        uint1024_t maximum_product = uint1024_t(binary_basis.modulus) * uint1024_t(prime_basis.modulus) /
                                     uint1024_t(static_cast<uint64_t>(num_products));
        // TODO: compute square root (the following is a lower bound, so good for the CRT use)
        uint64_t maximum_product_bits = maximum_product.get_msb() - 1;
        return (uint512_t(1) << (maximum_product_bits >> 1)) - uint512_t(1);
    }

    static constexpr uint1024_t get_maximum_crt_product()
    {
        uint1024_t maximum_product = uint1024_t(binary_basis.modulus) * uint1024_t(prime_basis.modulus);
        return maximum_product;
    }

    /**
     * @brief Compute the maximum number of bits for quotient range proof to protect against CRT underflow
     *
     * @param remainders_max Maximum sizes of resulting remainders
     * @return Desired length of range proof
     */
    static size_t get_quotient_max_bits(const std::vector<uint1024_t>& remainders_max)
    {
        // find q_max * p + ...remainders_max < nT
        uint1024_t base = get_maximum_crt_product();
        for (const auto& r : remainders_max) {
            base -= r;
        }
        base /= modulus_u512;
        return static_cast<size_t>(base.get_msb() - 1);
    }

    /**
     * Check that the maximum value of a bigfield product with added values overflows ctf modulus.
     *
     * @param a_max multiplicand maximum value
     * @param b_max multiplier maximum value
     * @param to_add vector of field elements to be added
     *
     * @return true if there is an overflow, false otherwise
     **/
    static bool mul_product_overflows_crt_modulus(const uint1024_t& a_max,
                                                  const uint1024_t& b_max,
                                                  const std::vector<bigfield>& to_add)
    {
        uint1024_t product = a_max * b_max;
        uint1024_t add_term;
        for (const auto& add : to_add) {
            add_term += add.get_maximum_value();
        }
        constexpr uint1024_t maximum_default_bigint = uint1024_t(1) << (NUM_LIMB_BITS * 6 + NUM_LAST_LIMB_BITS * 2);

        // check that the add terms alone cannot overflow the crt modulus. v. unlikely so just forbid circuits that
        // trigger this case
        ASSERT(add_term + maximum_default_bigint < get_maximum_crt_product());
        return ((product + add_term) >= get_maximum_crt_product());
    }

    /**
     * Check that the maximum value of a sum of bigfield productc with added values overflows ctf modulus.
     *
     * @param as_max Vector of multiplicands' maximum values
     * @param b_max Vector of multipliers' maximum values
     * @param to_add Vector of field elements to be added
     *
     * @return true if there is an overflow, false otherwise
     **/
    static bool mul_product_overflows_crt_modulus(const std::vector<uint512_t>& as_max,
                                                  const std::vector<uint512_t>& bs_max,
                                                  const std::vector<bigfield>& to_add)
    {
        std::vector<uint1024_t> products;
        ASSERT(as_max.size() == bs_max.size());
        // Computing individual products
        uint1024_t product_sum;
        uint1024_t add_term;
        for (size_t i = 0; i < as_max.size(); i++) {
            product_sum += uint1024_t(as_max[i]) * uint1024_t(bs_max[i]);
        }
        for (const auto& add : to_add) {
            add_term += add.get_maximum_value();
        }
        constexpr uint1024_t maximum_default_bigint = uint1024_t(1) << (NUM_LIMB_BITS * 6 + NUM_LAST_LIMB_BITS * 2);

        // check that the add terms alone cannot overflow the crt modulus. v. unlikely so just forbid circuits that
        // trigger this case
        ASSERT(add_term + maximum_default_bigint < get_maximum_crt_product());
        return ((product_sum + add_term) >= get_maximum_crt_product());
    }
    // static bool mul_quotient_crt_check(const uint1024_t& q, const std::vector<uint1024_t>& remainders)
    // {
    //     uint1024_t product = (q * modulus_u512);
    //     for (const auto& add : remainders) {
    //         product += add;
    //     }
    //     std::cout << "product = " << product << std::endl;
    //     std::cout << "crt product = " << get_maximum_crt_product() << std::endl;

    //     if (product >= get_maximum_crt_product()) {
    //         count++;
    //         std::cout << "count = " << count << std::endl;
    //     }
    //     return (product >= get_maximum_crt_product());
    // }
    // a (currently generous) upper bound on the log of number of fr additions in any of the class operations
    static constexpr uint64_t MAX_ADDITION_LOG = 10;
    // the rationale of the expression is we should not overflow Fr when applying any bigfield operation (e.g. *) and
    // starting with this max limb size
    static constexpr uint64_t MAX_UNREDUCED_LIMB_SIZE =
        (barretenberg::fr::modulus.get_msb() + 1) / 2 - MAX_ADDITION_LOG;

    static constexpr uint256_t get_maximum_unreduced_limb_value() { return uint256_t(1) << MAX_UNREDUCED_LIMB_SIZE; }

    static_assert(MAX_UNREDUCED_LIMB_SIZE < (NUM_LIMB_BITS * 2));
    Composer* context;
    mutable Limb binary_basis_limbs[4];
    mutable field_t<Composer> prime_basis_limb;

  private:
    static std::pair<uint512_t, uint512_t> compute_quotient_remainder_values(const bigfield& a,
                                                                             const bigfield& b,
                                                                             const std::vector<bigfield>& to_add);
    /**
     * @brief Compute the maximum possible value of quotient of a*b+\sum(to_add)
     *
     * @param as Multiplicands
     * @param bs Multipliers
     * @param to_add Added elements
     * @return uint512_t The maximum value of quotient
     */
    static uint512_t compute_maximum_quotient_value(const std::vector<uint512_t>& as,
                                                    const std::vector<uint512_t>& bs,
                                                    const std::vector<uint512_t>& to_add);

    /**
     * @brief Check for 2 conditions (CRT modulus is overflown or the maximum quotient doesn't fit into range proof).
     * Also returns the length of quotient's range proof if there is no need to reduce.
     *
     * @param as_max Vector of left multiplicands' maximum values
     * @param bs_max Vector of right multiplicands' maximum values
     * @param to_add Vector of added bigfield values
     * @return <true, 0> if we need to reduce the product;
     * <false, The length of quotient range proof> if there is no need to reduce the product.
     */
    static std::pair<bool, size_t> get_quotient_reduction_info(const std::vector<uint512_t>& as_max,
                                                               const std::vector<uint512_t>& bs_max,
                                                               const std::vector<bigfield>& to_add,
                                                               const std::vector<uint1024_t>& remainders_max = {
                                                                   DEFAULT_MAXIMUM_REMAINDER });

    static void unsafe_evaluate_multiply_add(const bigfield& left,
                                             const bigfield& right_mul,
                                             const std::vector<bigfield>& to_add,
                                             const bigfield& quotient,
                                             const std::vector<bigfield>& remainders);

    static void unsafe_evaluate_multiple_multiply_add(const std::vector<bigfield>& input_left,
                                                      const std::vector<bigfield>& input_right,
                                                      const std::vector<bigfield>& to_add,
                                                      const bigfield& input_quotient,
                                                      const std::vector<bigfield>& input_remainders);

    static void unsafe_evaluate_square_add(const bigfield& left,
                                           const std::vector<bigfield>& to_add,
                                           const bigfield& quotient,
                                           const bigfield& remainder);

    static void evaluate_product(const bigfield& left,
                                 const bigfield& right,
                                 const bigfield& quotient,
                                 const bigfield& remainder);
    void reduction_check(const size_t num_products = 1) const;

}; // namespace stdlib

template <typename C, typename T> inline std::ostream& operator<<(std::ostream& os, bigfield<T, C> const& v)
{
    return os << v.get_value();
}

} // namespace stdlib
} // namespace plonk

#include "bigfield_impl.hpp"