#pragma once
#include "barretenberg/common/assert.hpp"
#include "barretenberg/common/compiler_hints.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include <array>
#include <cstdint>
#include <iostream>
#include <random>
#include <span>

#ifndef DISABLE_SHENANIGANS
#ifdef __BMI2__
#define BBERG_NO_ASM 0
#else
#define BBERG_NO_ASM 1
#endif
#else
#define BBERG_NO_ASM 1
#endif

namespace bb {
template <class Params_> struct alignas(32) field {
  public:
    using View = field;
    using Params = Params_;
    using in_buf = const uint8_t*;
    using vec_in_buf = const uint8_t*;
    using out_buf = uint8_t*;
    using vec_out_buf = uint8_t**;

    // We don't initialize data in the default constructor since we'd lose a lot of time on huge array initializations.
    // Other alternatives have been noted, such as casting to get around constructors where they matter,
    // however it is felt that sanitizer tools (e.g. MSAN) can detect garbage well, whereas doing
    // hacky casts where needed would require rework to critical algos like MSM, FFT, Sumcheck.
    // Instead, the recommended solution is use an explicit {} where initialization is important:
    //  field f; // not initialized
    //  field f{}; // zero-initialized
    //  std::array<field, N> arr; // not initialized, good for huge N
    //  std::array<field, N> arr {}; // zero-initialized, preferable for moderate N
    field() = default;

    constexpr field(const numeric::uint256_t& input) noexcept
        : data{ input.data[0], input.data[1], input.data[2], input.data[3] }
    {
        self_to_montgomery_form();
    }

    // NOLINTNEXTLINE (unsigned long is platform dependent, which we want in this case)
    constexpr field(const unsigned long input) noexcept
        : data{ input, 0, 0, 0 }
    {
        self_to_montgomery_form();
    }

    constexpr field(const unsigned int input) noexcept
        : data{ input, 0, 0, 0 }
    {
        self_to_montgomery_form();
    }

    // NOLINTNEXTLINE (unsigned long long is platform dependent, which we want in this case)
    constexpr field(const unsigned long long input) noexcept
        : data{ input, 0, 0, 0 }
    {
        self_to_montgomery_form();
    }

    constexpr field(const int input) noexcept
        : data{ 0, 0, 0, 0 }
    {
        if (input < 0) {
            data[0] = static_cast<uint64_t>(-input);
            data[1] = 0;
            data[2] = 0;
            data[3] = 0;
            self_to_montgomery_form();
            self_neg();
            self_reduce_once();
        } else {
            data[0] = static_cast<uint64_t>(input);
            data[1] = 0;
            data[2] = 0;
            data[3] = 0;
            self_to_montgomery_form();
        }
    }

    constexpr field(const uint64_t a, const uint64_t b, const uint64_t c, const uint64_t d) noexcept
        : data{ a, b, c, d } {};

    /**
     * @brief Convert a 512-bit big integer into a field element.
     *
     * @details Used for deriving field elements from random values. 512-bits prevents biased output as 2^512>>modulus
     *
     */
    constexpr explicit field(const uint512_t& input) noexcept
    {
        uint256_t value = (input % modulus).lo;
        data[0] = value.data[0];
        data[1] = value.data[1];
        data[2] = value.data[2];
        data[3] = value.data[3];
        self_to_montgomery_form();
    }

    constexpr explicit field(std::string input) noexcept
    {
        uint256_t value(input);
        *this = field(value);
    }

    constexpr explicit operator uint32_t() const
    {
        field out = from_montgomery_form();
        return static_cast<uint32_t>(out.data[0]);
    }

    constexpr explicit operator uint64_t() const
    {
        field out = from_montgomery_form();
        return out.data[0];
    }

    constexpr explicit operator uint128_t() const
    {
        field out = from_montgomery_form();
        uint128_t lo = out.data[0];
        uint128_t hi = out.data[1];
        return (hi << 64) | lo;
    }

    constexpr operator uint256_t() const noexcept
    {
        field out = from_montgomery_form();
        return uint256_t(out.data[0], out.data[1], out.data[2], out.data[3]);
    }

    [[nodiscard]] constexpr uint256_t uint256_t_no_montgomery_conversion() const noexcept
    {
        return { data[0], data[1], data[2], data[3] };
    }

    constexpr field(const field& other) noexcept = default;
    constexpr field(field&& other) noexcept = default;
    constexpr field& operator=(const field& other) noexcept = default;
    constexpr field& operator=(field&& other) noexcept = default;
    constexpr ~field() noexcept = default;
    alignas(32) uint64_t data[4]; // NOLINT

    static constexpr uint256_t modulus =
        uint256_t{ Params::modulus_0, Params::modulus_1, Params::modulus_2, Params::modulus_3 };

    static constexpr field cube_root_of_unity()
    {
        // endomorphism i.e. lambda * [P] = (beta * x, y)
        if constexpr (Params::cube_root_0 != 0) {
            constexpr field result{
                Params::cube_root_0, Params::cube_root_1, Params::cube_root_2, Params::cube_root_3
            };
            return result;
        } else {
            constexpr field two_inv = field(2).invert();
            constexpr field numerator = (-field(3)).sqrt() - field(1);
            constexpr field result = two_inv * numerator;
            return result;
        }
    }

    static constexpr field zero() { return field(0, 0, 0, 0); }
    static constexpr field neg_one() { return -field(1); }
    static constexpr field one() { return field(1); }

    static constexpr field external_coset_generator()
    {
        const field result{
            Params::coset_generators_0[7],
            Params::coset_generators_1[7],
            Params::coset_generators_2[7],
            Params::coset_generators_3[7],
        };
        return result;
    }

    static constexpr field tag_coset_generator()
    {
        const field result{
            Params::coset_generators_0[6],
            Params::coset_generators_1[6],
            Params::coset_generators_2[6],
            Params::coset_generators_3[6],
        };
        return result;
    }

    static constexpr field coset_generator(const size_t idx)
    {
        ASSERT(idx < 7);
        const field result{
            Params::coset_generators_0[idx],
            Params::coset_generators_1[idx],
            Params::coset_generators_2[idx],
            Params::coset_generators_3[idx],
        };
        return result;
    }

    BBERG_INLINE constexpr field operator*(const field& other) const noexcept;
    BBERG_INLINE constexpr field operator+(const field& other) const noexcept;
    BBERG_INLINE constexpr field operator-(const field& other) const noexcept;
    BBERG_INLINE constexpr field operator-() const noexcept;
    constexpr field operator/(const field& other) const noexcept;

    // prefix increment (++x)
    BBERG_INLINE constexpr field operator++() noexcept;
    // postfix increment (x++)
    // NOLINTNEXTLINE
    BBERG_INLINE constexpr field operator++(int) noexcept;

    BBERG_INLINE constexpr field& operator*=(const field& other) noexcept;
    BBERG_INLINE constexpr field& operator+=(const field& other) noexcept;
    BBERG_INLINE constexpr field& operator-=(const field& other) noexcept;
    constexpr field& operator/=(const field& other) noexcept;

    // NOTE: comparison operators exist so that `field` is comparible with stl methods that require them.
    //       (e.g. std::sort)
    //       Finite fields do not have an explicit ordering, these should *NEVER* be used in algebraic algorithms.
    BBERG_INLINE constexpr bool operator>(const field& other) const noexcept;
    BBERG_INLINE constexpr bool operator<(const field& other) const noexcept;
    BBERG_INLINE constexpr bool operator==(const field& other) const noexcept;
    BBERG_INLINE constexpr bool operator!=(const field& other) const noexcept;

    BBERG_INLINE constexpr field to_montgomery_form() const noexcept;
    BBERG_INLINE constexpr field from_montgomery_form() const noexcept;

    BBERG_INLINE constexpr field sqr() const noexcept;
    BBERG_INLINE constexpr void self_sqr() noexcept;

    BBERG_INLINE constexpr field pow(const uint256_t& exponent) const noexcept;
    BBERG_INLINE constexpr field pow(uint64_t exponent) const noexcept;
    static constexpr uint256_t modulus_minus_two =
        uint256_t(Params::modulus_0 - 2ULL, Params::modulus_1, Params::modulus_2, Params::modulus_3);
    constexpr field invert() const noexcept;
    static void batch_invert(std::span<field> coeffs) noexcept;
    static void batch_invert(field* coeffs, size_t n) noexcept;
    /**
     * @brief Compute square root of the field element.
     *
     * @return <true, root> if the element is a quadratic remainder, <false, 0> if it's not
     */
    constexpr std::pair<bool, field> sqrt() const noexcept;

    BBERG_INLINE constexpr void self_neg() noexcept;

    BBERG_INLINE constexpr void self_to_montgomery_form() noexcept;
    BBERG_INLINE constexpr void self_from_montgomery_form() noexcept;

    BBERG_INLINE constexpr void self_conditional_negate(uint64_t predicate) noexcept;

    BBERG_INLINE constexpr field reduce_once() const noexcept;
    BBERG_INLINE constexpr void self_reduce_once() noexcept;

    BBERG_INLINE constexpr void self_set_msb() noexcept;
    [[nodiscard]] BBERG_INLINE constexpr bool is_msb_set() const noexcept;
    [[nodiscard]] BBERG_INLINE constexpr uint64_t is_msb_set_word() const noexcept;

    [[nodiscard]] BBERG_INLINE constexpr bool is_zero() const noexcept;

    static constexpr field get_root_of_unity(size_t subgroup_size) noexcept;

    static void serialize_to_buffer(const field& value, uint8_t* buffer) { write(buffer, value); }

    static field serialize_from_buffer(const uint8_t* buffer) { return from_buffer<field>(buffer); }

    [[nodiscard]] BBERG_INLINE std::vector<uint8_t> to_buffer() const { return ::to_buffer(*this); }

    struct wide_array {
        uint64_t data[8]; // NOLINT
    };
    BBERG_INLINE constexpr wide_array mul_512(const field& other) const noexcept;
    BBERG_INLINE constexpr wide_array sqr_512() const noexcept;

    BBERG_INLINE constexpr field conditionally_subtract_from_double_modulus(const uint64_t predicate) const noexcept
    {
        if (predicate != 0) {
            constexpr field p{
                twice_modulus.data[0], twice_modulus.data[1], twice_modulus.data[2], twice_modulus.data[3]
            };
            return p - *this;
        }
        return *this;
    }

    /**
     * For short Weierstrass curves y^2 = x^3 + b mod r, if there exists a cube root of unity mod r,
     * we can take advantage of an enodmorphism to decompose a 254 bit scalar into 2 128 bit scalars.
     * \beta = cube root of 1, mod q (q = order of fq)
     * \lambda = cube root of 1, mod r (r = order of fr)
     *
     * For a point P1 = (X, Y), where Y^2 = X^3 + b, we know that
     * the point P2 = (X * \beta, Y) is also a point on the curve
     * We can represent P2 as a scalar multiplication of P1, where P2 = \lambda * P1
     *
     * For a generic multiplication of P1 by a 254 bit scalar k, we can decompose k
     * into 2 127 bit scalars (k1, k2), such that k = k1 - (k2 * \lambda)
     *
     * We can now represent (k * P1) as (k1 * P1) - (k2 * P2), where P2 = (X * \beta, Y).
     * As k1, k2 have half the bit length of k, we have reduced the number of loop iterations of our
     * scalar multiplication algorithm in half
     *
     * To find k1, k2, We use the extended euclidean algorithm to find 4 short scalars [a1, a2], [b1, b2] such that
     * modulus = (a1 * b2) - (b1 * a2)
     * We then compute scalars c1 = round(b2 * k / r), c2 = round(b1 * k / r), where
     * k1 = (c1 * a1) + (c2 * a2), k2 = -((c1 * b1) + (c2 * b2))
     * We pre-compute scalars g1 = (2^256 * b1) / n, g2 = (2^256 * b2) / n, to avoid having to perform long division
     * on 512-bit scalars
     **/
    static void split_into_endomorphism_scalars(const field& k, field& k1, field& k2)
    {
        // if the modulus is a 256-bit integer, we need to use a basis where g1, g2 have been shifted by 2^384
        if constexpr (Params::modulus_3 >= 0x4000000000000000ULL) {
            split_into_endomorphism_scalars_384(k, k1, k2);
            return;
        }
        field input = k.reduce_once();

        constexpr field endo_g1 = { Params::endo_g1_lo, Params::endo_g1_mid, Params::endo_g1_hi, 0 };

        constexpr field endo_g2 = { Params::endo_g2_lo, Params::endo_g2_mid, 0, 0 };

        constexpr field endo_minus_b1 = { Params::endo_minus_b1_lo, Params::endo_minus_b1_mid, 0, 0 };

        constexpr field endo_b2 = { Params::endo_b2_lo, Params::endo_b2_mid, 0, 0 };

        // compute c1 = (g2 * k) >> 256
        wide_array c1 = endo_g2.mul_512(input);
        // compute c2 = (g1 * k) >> 256
        wide_array c2 = endo_g1.mul_512(input);

        // (the bit shifts are implicit, as we only utilize the high limbs of c1, c2

        field c1_hi = {
            c1.data[4], c1.data[5], c1.data[6], c1.data[7]
        }; // *(field*)((uintptr_t)(&c1) + (4 * sizeof(uint64_t)));
        field c2_hi = {
            c2.data[4], c2.data[5], c2.data[6], c2.data[7]
        }; // *(field*)((uintptr_t)(&c2) + (4 * sizeof(uint64_t)));

        // compute q1 = c1 * -b1
        wide_array q1 = c1_hi.mul_512(endo_minus_b1);
        // compute q2 = c2 * b2
        wide_array q2 = c2_hi.mul_512(endo_b2);

        // FIX: Avoid using 512-bit multiplication as its not necessary.
        // c1_hi, c2_hi can be uint256_t's and the final result (without montgomery reduction)
        // could be casted to a field.
        field q1_lo{ q1.data[0], q1.data[1], q1.data[2], q1.data[3] };
        field q2_lo{ q2.data[0], q2.data[1], q2.data[2], q2.data[3] };

        field t1 = (q2_lo - q1_lo).reduce_once();
        field beta = cube_root_of_unity();
        field t2 = (t1 * beta + input).reduce_once();
        k2.data[0] = t1.data[0];
        k2.data[1] = t1.data[1];
        k1.data[0] = t2.data[0];
        k1.data[1] = t2.data[1];
    }

    static void split_into_endomorphism_scalars_384(const field& input, field& k1_out, field& k2_out)
    {

        constexpr field minus_b1f{
            Params::endo_minus_b1_lo,
            Params::endo_minus_b1_mid,
            0,
            0,
        };
        constexpr field b2f{
            Params::endo_b2_lo,
            Params::endo_b2_mid,
            0,
            0,
        };
        constexpr uint256_t g1{
            Params::endo_g1_lo,
            Params::endo_g1_mid,
            Params::endo_g1_hi,
            Params::endo_g1_hihi,
        };
        constexpr uint256_t g2{
            Params::endo_g2_lo,
            Params::endo_g2_mid,
            Params::endo_g2_hi,
            Params::endo_g2_hihi,
        };

        field kf = input.reduce_once();
        uint256_t k{ kf.data[0], kf.data[1], kf.data[2], kf.data[3] };

        uint512_t c1 = (uint512_t(k) * static_cast<uint512_t>(g1)) >> 384;
        uint512_t c2 = (uint512_t(k) * static_cast<uint512_t>(g2)) >> 384;

        field c1f{ c1.lo.data[0], c1.lo.data[1], c1.lo.data[2], c1.lo.data[3] };
        field c2f{ c2.lo.data[0], c2.lo.data[1], c2.lo.data[2], c2.lo.data[3] };

        c1f.self_to_montgomery_form();
        c2f.self_to_montgomery_form();
        c1f = c1f * minus_b1f;
        c2f = c2f * b2f;
        field r2f = c1f - c2f;
        field beta = cube_root_of_unity();
        field r1f = input.reduce_once() - r2f * beta;
        k1_out = r1f;
        k2_out = -r2f;
    }

    // static constexpr auto coset_generators = compute_coset_generators();
    // static constexpr std::array<field, 15> coset_generators = compute_coset_generators((1 << 30U));

    friend std::ostream& operator<<(std::ostream& os, const field& a)
    {
        field out = a.from_montgomery_form();
        std::ios_base::fmtflags f(os.flags());
        os << std::hex << "0x" << std::setfill('0') << std::setw(16) << out.data[3] << std::setw(16) << out.data[2]
           << std::setw(16) << out.data[1] << std::setw(16) << out.data[0];
        os.flags(f);
        return os;
    }

    BBERG_INLINE static void __copy(const field& a, field& r) noexcept { r = a; } // NOLINT
    BBERG_INLINE static void __swap(field& src, field& dest) noexcept             // NOLINT
    {
        field T = dest;
        dest = src;
        src = T;
    }

    static field random_element(numeric::random::Engine* engine = nullptr) noexcept;

    static constexpr field multiplicative_generator() noexcept;

    // For serialization
    void msgpack_pack(auto& packer) const;
    void msgpack_unpack(auto o);
    void msgpack_schema(auto& packer) const { packer.pack_alias(Params::schema_name, "bin32"); }

  private:
    static constexpr uint256_t twice_modulus = modulus + modulus;
    static constexpr uint256_t not_modulus = -modulus;
    static constexpr uint256_t twice_not_modulus = -twice_modulus;

    struct wnaf_table {
        uint8_t windows[64]; // NOLINT

        constexpr wnaf_table(const uint256_t& target)
            : windows{
                static_cast<uint8_t>(target.data[0] & 15),         static_cast<uint8_t>((target.data[0] >> 4) & 15),
                static_cast<uint8_t>((target.data[0] >> 8) & 15),  static_cast<uint8_t>((target.data[0] >> 12) & 15),
                static_cast<uint8_t>((target.data[0] >> 16) & 15), static_cast<uint8_t>((target.data[0] >> 20) & 15),
                static_cast<uint8_t>((target.data[0] >> 24) & 15), static_cast<uint8_t>((target.data[0] >> 28) & 15),
                static_cast<uint8_t>((target.data[0] >> 32) & 15), static_cast<uint8_t>((target.data[0] >> 36) & 15),
                static_cast<uint8_t>((target.data[0] >> 40) & 15), static_cast<uint8_t>((target.data[0] >> 44) & 15),
                static_cast<uint8_t>((target.data[0] >> 48) & 15), static_cast<uint8_t>((target.data[0] >> 52) & 15),
                static_cast<uint8_t>((target.data[0] >> 56) & 15), static_cast<uint8_t>((target.data[0] >> 60) & 15),
                static_cast<uint8_t>(target.data[1] & 15),         static_cast<uint8_t>((target.data[1] >> 4) & 15),
                static_cast<uint8_t>((target.data[1] >> 8) & 15),  static_cast<uint8_t>((target.data[1] >> 12) & 15),
                static_cast<uint8_t>((target.data[1] >> 16) & 15), static_cast<uint8_t>((target.data[1] >> 20) & 15),
                static_cast<uint8_t>((target.data[1] >> 24) & 15), static_cast<uint8_t>((target.data[1] >> 28) & 15),
                static_cast<uint8_t>((target.data[1] >> 32) & 15), static_cast<uint8_t>((target.data[1] >> 36) & 15),
                static_cast<uint8_t>((target.data[1] >> 40) & 15), static_cast<uint8_t>((target.data[1] >> 44) & 15),
                static_cast<uint8_t>((target.data[1] >> 48) & 15), static_cast<uint8_t>((target.data[1] >> 52) & 15),
                static_cast<uint8_t>((target.data[1] >> 56) & 15), static_cast<uint8_t>((target.data[1] >> 60) & 15),
                static_cast<uint8_t>(target.data[2] & 15),         static_cast<uint8_t>((target.data[2] >> 4) & 15),
                static_cast<uint8_t>((target.data[2] >> 8) & 15),  static_cast<uint8_t>((target.data[2] >> 12) & 15),
                static_cast<uint8_t>((target.data[2] >> 16) & 15), static_cast<uint8_t>((target.data[2] >> 20) & 15),
                static_cast<uint8_t>((target.data[2] >> 24) & 15), static_cast<uint8_t>((target.data[2] >> 28) & 15),
                static_cast<uint8_t>((target.data[2] >> 32) & 15), static_cast<uint8_t>((target.data[2] >> 36) & 15),
                static_cast<uint8_t>((target.data[2] >> 40) & 15), static_cast<uint8_t>((target.data[2] >> 44) & 15),
                static_cast<uint8_t>((target.data[2] >> 48) & 15), static_cast<uint8_t>((target.data[2] >> 52) & 15),
                static_cast<uint8_t>((target.data[2] >> 56) & 15), static_cast<uint8_t>((target.data[2] >> 60) & 15),
                static_cast<uint8_t>(target.data[3] & 15),         static_cast<uint8_t>((target.data[3] >> 4) & 15),
                static_cast<uint8_t>((target.data[3] >> 8) & 15),  static_cast<uint8_t>((target.data[3] >> 12) & 15),
                static_cast<uint8_t>((target.data[3] >> 16) & 15), static_cast<uint8_t>((target.data[3] >> 20) & 15),
                static_cast<uint8_t>((target.data[3] >> 24) & 15), static_cast<uint8_t>((target.data[3] >> 28) & 15),
                static_cast<uint8_t>((target.data[3] >> 32) & 15), static_cast<uint8_t>((target.data[3] >> 36) & 15),
                static_cast<uint8_t>((target.data[3] >> 40) & 15), static_cast<uint8_t>((target.data[3] >> 44) & 15),
                static_cast<uint8_t>((target.data[3] >> 48) & 15), static_cast<uint8_t>((target.data[3] >> 52) & 15),
                static_cast<uint8_t>((target.data[3] >> 56) & 15), static_cast<uint8_t>((target.data[3] >> 60) & 15)
            }
        {}
    };

    BBERG_INLINE static constexpr std::pair<uint64_t, uint64_t> mul_wide(uint64_t a, uint64_t b) noexcept;

    BBERG_INLINE static constexpr uint64_t mac(
        uint64_t a, uint64_t b, uint64_t c, uint64_t carry_in, uint64_t& carry_out) noexcept;

    BBERG_INLINE static constexpr void mac(
        uint64_t a, uint64_t b, uint64_t c, uint64_t carry_in, uint64_t& out, uint64_t& carry_out) noexcept;

    BBERG_INLINE static constexpr uint64_t mac_mini(uint64_t a, uint64_t b, uint64_t c, uint64_t& out) noexcept;

    BBERG_INLINE static constexpr void mac_mini(
        uint64_t a, uint64_t b, uint64_t c, uint64_t& out, uint64_t& carry_out) noexcept;

    BBERG_INLINE static constexpr uint64_t mac_discard_lo(uint64_t a, uint64_t b, uint64_t c) noexcept;

    BBERG_INLINE static constexpr uint64_t addc(uint64_t a,
                                                uint64_t b,
                                                uint64_t carry_in,
                                                uint64_t& carry_out) noexcept;

    BBERG_INLINE static constexpr uint64_t sbb(uint64_t a,
                                               uint64_t b,
                                               uint64_t borrow_in,
                                               uint64_t& borrow_out) noexcept;

    BBERG_INLINE static constexpr uint64_t square_accumulate(uint64_t a,
                                                             uint64_t b,
                                                             uint64_t c,
                                                             uint64_t carry_in_lo,
                                                             uint64_t carry_in_hi,
                                                             uint64_t& carry_lo,
                                                             uint64_t& carry_hi) noexcept;
    BBERG_INLINE constexpr field reduce() const noexcept;
    BBERG_INLINE constexpr field add(const field& other) const noexcept;
    BBERG_INLINE constexpr field subtract(const field& other) const noexcept;
    BBERG_INLINE constexpr field subtract_coarse(const field& other) const noexcept;
    BBERG_INLINE constexpr field montgomery_mul(const field& other) const noexcept;
    BBERG_INLINE constexpr field montgomery_mul_big(const field& other) const noexcept;
    BBERG_INLINE constexpr field montgomery_square() const noexcept;

#if (BBERG_NO_ASM == 0)
    BBERG_INLINE static field asm_mul(const field& a, const field& b) noexcept;
    BBERG_INLINE static field asm_sqr(const field& a) noexcept;
    BBERG_INLINE static field asm_add(const field& a, const field& b) noexcept;
    BBERG_INLINE static field asm_sub(const field& a, const field& b) noexcept;
    BBERG_INLINE static field asm_mul_with_coarse_reduction(const field& a, const field& b) noexcept;
    BBERG_INLINE static field asm_sqr_with_coarse_reduction(const field& a) noexcept;
    BBERG_INLINE static field asm_add_with_coarse_reduction(const field& a, const field& b) noexcept;
    BBERG_INLINE static field asm_sub_with_coarse_reduction(const field& a, const field& b) noexcept;
    BBERG_INLINE static field asm_add_without_reduction(const field& a, const field& b) noexcept;
    BBERG_INLINE static void asm_self_sqr(const field& a) noexcept;
    BBERG_INLINE static void asm_self_add(const field& a, const field& b) noexcept;
    BBERG_INLINE static void asm_self_sub(const field& a, const field& b) noexcept;
    BBERG_INLINE static void asm_self_mul_with_coarse_reduction(const field& a, const field& b) noexcept;
    BBERG_INLINE static void asm_self_sqr_with_coarse_reduction(const field& a) noexcept;
    BBERG_INLINE static void asm_self_add_with_coarse_reduction(const field& a, const field& b) noexcept;
    BBERG_INLINE static void asm_self_sub_with_coarse_reduction(const field& a, const field& b) noexcept;
    BBERG_INLINE static void asm_self_add_without_reduction(const field& a, const field& b) noexcept;

    BBERG_INLINE static void asm_conditional_negate(field& r, uint64_t predicate) noexcept;
    BBERG_INLINE static field asm_reduce_once(const field& a) noexcept;
    BBERG_INLINE static void asm_self_reduce_once(const field& a) noexcept;
    static constexpr uint64_t zero_reference = 0x00ULL;
#endif
    static constexpr size_t COSET_GENERATOR_SIZE = 15;
    constexpr field tonelli_shanks_sqrt() const noexcept;
    static constexpr size_t primitive_root_log_size() noexcept;
    static constexpr std::array<field, COSET_GENERATOR_SIZE> compute_coset_generators() noexcept;

#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    static constexpr uint128_t lo_mask = 0xffffffffffffffffUL;
#endif
};

template <typename B, typename Params> void read(B& it, field<Params>& value)
{
    using serialize::read;
    field<Params> result{ 0, 0, 0, 0 };
    read(it, result.data[3]);
    read(it, result.data[2]);
    read(it, result.data[1]);
    read(it, result.data[0]);
    value = result.to_montgomery_form();
}
template <typename B, typename Params> void write(B& buf, field<Params> const& value)
{
    using serialize::write;
    const field input = value.from_montgomery_form();
    write(buf, input.data[3]);
    write(buf, input.data[2]);
    write(buf, input.data[1]);
    write(buf, input.data[0]);
}

} // namespace bb
