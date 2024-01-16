#pragma once
#include "./element.hpp"
#include "barretenberg/crypto/blake3s/blake3s.hpp"
#include "barretenberg/crypto/keccak/keccak.hpp"

namespace bb::group_elements {
template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T>::affine_element(const Fq& a, const Fq& b) noexcept
    : x(a)
    , y(b)
{}

template <class Fq, class Fr, class T>
template <typename BaseField, typename CompileTimeEnabled>
constexpr affine_element<Fq, Fr, T> affine_element<Fq, Fr, T>::from_compressed(const uint256_t& compressed) noexcept
{
    uint256_t x_coordinate = compressed;
    x_coordinate.data[3] = x_coordinate.data[3] & (~0x8000000000000000ULL);
    bool y_bit = compressed.get_bit(255);

    Fq x = Fq(x_coordinate);
    Fq y2 = (x.sqr() * x + T::b);
    if constexpr (T::has_a) {
        y2 += (x * T::a);
    }
    auto [is_quadratic_remainder, y] = y2.sqrt();
    if (!is_quadratic_remainder) {
        return affine_element(Fq::zero(), Fq::zero());
    }
    if (uint256_t(y).get_bit(0) != y_bit) {
        y = -y;
    }

    return affine_element<Fq, Fr, T>(x, y);
}

template <class Fq, class Fr, class T>
template <typename BaseField, typename CompileTimeEnabled>
constexpr std::array<affine_element<Fq, Fr, T>, 2> affine_element<Fq, Fr, T>::from_compressed_unsafe(
    const uint256_t& compressed) noexcept
{
    auto get_y_coordinate = [](const uint256_t& x_coordinate) {
        Fq x = Fq(x_coordinate);
        Fq y2 = (x.sqr() * x + T::b);
        if constexpr (T::has_a) {
            y2 += (x * T::a);
        }
        return y2.sqrt();
    };

    uint256_t x_1 = compressed;
    uint256_t x_2 = compressed + Fr::modulus;
    auto [is_quadratic_remainder_1, y_1] = get_y_coordinate(x_1);
    auto [is_quadratic_remainder_2, y_2] = get_y_coordinate(x_2);

    auto output_1 = is_quadratic_remainder_1 ? affine_element<Fq, Fr, T>(Fq(x_1), y_1)
                                             : affine_element<Fq, Fr, T>(Fq::zero(), Fq::zero());
    auto output_2 = is_quadratic_remainder_2 ? affine_element<Fq, Fr, T>(Fq(x_2), y_2)
                                             : affine_element<Fq, Fr, T>(Fq::zero(), Fq::zero());

    return { output_1, output_2 };
}

template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T> affine_element<Fq, Fr, T>::operator+(
    const affine_element<Fq, Fr, T>& other) const noexcept
{
    return affine_element(element<Fq, Fr, T>(*this) + element<Fq, Fr, T>(other));
}

template <class Fq, class Fr, class T>
template <typename BaseField, typename CompileTimeEnabled>

constexpr uint256_t affine_element<Fq, Fr, T>::compress() const noexcept
{
    uint256_t out(x);
    if (uint256_t(y).get_bit(0)) {
        out.data[3] = out.data[3] | 0x8000000000000000ULL;
    }
    return out;
}

template <class Fq, class Fr, class T> affine_element<Fq, Fr, T> affine_element<Fq, Fr, T>::infinity()
{
    affine_element e;
    e.self_set_infinity();
    return e;
}

template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T> affine_element<Fq, Fr, T>::set_infinity() const noexcept
{
    affine_element result(*this);
    result.self_set_infinity();
    return result;
}

template <class Fq, class Fr, class T> constexpr void affine_element<Fq, Fr, T>::self_set_infinity() noexcept
{
    if constexpr (Fq::modulus.data[3] >= 0x4000000000000000ULL) {
        // We set the value of x equal to modulus to represent inifinty
        x.data[0] = Fq::modulus.data[0];
        x.data[1] = Fq::modulus.data[1];
        x.data[2] = Fq::modulus.data[2];
        x.data[3] = Fq::modulus.data[3];

    } else {
        x.self_set_msb();
    }
}

template <class Fq, class Fr, class T> constexpr bool affine_element<Fq, Fr, T>::is_point_at_infinity() const noexcept
{
    if constexpr (Fq::modulus.data[3] >= 0x4000000000000000ULL) {
        // We check if the value of x is equal to modulus to represent inifinty
        return ((x.data[0] ^ Fq::modulus.data[0]) | (x.data[1] ^ Fq::modulus.data[1]) |
                (x.data[2] ^ Fq::modulus.data[2]) | (x.data[3] ^ Fq::modulus.data[3])) == 0;

    } else {
        return (x.is_msb_set());
    }
}

template <class Fq, class Fr, class T> constexpr bool affine_element<Fq, Fr, T>::on_curve() const noexcept
{
    if (is_point_at_infinity()) {
        return true;
    }
    Fq xxx = x.sqr() * x + T::b;
    Fq yy = y.sqr();
    if constexpr (T::has_a) {
        xxx += (x * T::a);
    }
    return (xxx == yy);
}

template <class Fq, class Fr, class T>
constexpr bool affine_element<Fq, Fr, T>::operator==(const affine_element& other) const noexcept
{
    bool this_is_infinity = is_point_at_infinity();
    bool other_is_infinity = other.is_point_at_infinity();
    bool both_infinity = this_is_infinity && other_is_infinity;
    bool only_one_is_infinity = this_is_infinity != other_is_infinity;
    return !only_one_is_infinity && (both_infinity || ((x == other.x) && (y == other.y)));
}

/**
 * Comparison operators (for std::sort)
 *
 * @details CAUTION!! Don't use this operator. It has no meaning other than for use by std::sort.
 **/
template <class Fq, class Fr, class T>
constexpr bool affine_element<Fq, Fr, T>::operator>(const affine_element& other) const noexcept
{
    // We are setting point at infinity to always be the lowest element
    if (is_point_at_infinity()) {
        return false;
    }
    if (other.is_point_at_infinity()) {
        return true;
    }

    if (x > other.x) {
        return true;
    }
    if (x == other.x && y > other.y) {
        return true;
    }
    return false;
}

template <class Fq, class Fr, class T>
constexpr std::optional<affine_element<Fq, Fr, T>> affine_element<Fq, Fr, T>::derive_from_x_coordinate(
    const Fq& x, bool sign_bit) noexcept
{
    auto yy = x.sqr() * x + T::b;
    if constexpr (T::has_a) {
        yy += (x * T::a);
    }
    auto [found_root, y] = yy.sqrt();

    if (found_root) {
        if (uint256_t(y).get_bit(0) != sign_bit) {
            y = -y;
        }
        return affine_element(x, y);
    }
    return std::nullopt;
}

/**
 * @brief Hash a seed buffer into a point
 *
 * @details ALGORITHM DESCRIPTION:
 *          1. Initialize unsigned integer `attempt_count = 0`
 *          2. Copy seed into a buffer whose size is 2 bytes greater than `seed` (initialized to 0)
 *          3. Interpret `attempt_count` as a byte and write into buffer at [buffer.size() - 2]
 *          4. Compute Blake3s hash of buffer
 *          5. Set the end byte of the buffer to `1`
 *          6. Compute Blake3s hash of buffer
 *          7. Interpret the two hash outputs as the high / low 256 bits of a 512-bit integer (big-endian)
 *          8. Derive x-coordinate of point by reducing the 512-bit integer modulo the curve's field modulus (Fq)
 *          9. Compute y^2 from the curve formula y^2 = x^3 + ax + b (a, b are curve params. for BN254, a = 0, b = 3)
 *          10. IF y^2 IS NOT A QUADRATIC RESIDUE
 *              10a. increment `attempt_count` by 1 and go to step 2
 *          11. IF y^2 IS A QUADRATIC RESIDUE
 *              11a. derive y coordinate via y = sqrt(y)
 *              11b. Interpret most significant bit of 512-bit integer as a 'parity' bit
 *              11c. If parity bit is set AND y's most significant bit is not set, invert y
 *              11d. If parity bit is not set AND y's most significant bit is set, invert y
 *              N.B. last 2 steps are because the sqrt() algorithm can return 2 values,
 *                   we need to a way to canonically distinguish between these 2 values and select a "preferred" one
 *              11e. return (x, y)
 *
 * @note This algorihm is constexpr: we can hash-to-curve (and derive generators) at compile-time!
 * @tparam Fq
 * @tparam Fr
 * @tparam T
 * @param seed Bytes that uniquely define the point being generated
 * @param attempt_count
 * @return constexpr affine_element<Fq, Fr, T>
 */
template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T> affine_element<Fq, Fr, T>::hash_to_curve(const std::vector<uint8_t>& seed,
                                                                             uint8_t attempt_count) noexcept
    requires SupportsHashToCurve<T>

{
    std::vector<uint8_t> target_seed(seed);
    // expand by 2 bytes to cover incremental hash attempts
    const size_t seed_size = seed.size();
    for (size_t i = 0; i < 2; ++i) {
        target_seed.push_back(0);
    }
    target_seed[seed_size] = attempt_count;
    target_seed[seed_size + 1] = 0;
    const auto hash_hi = blake3::blake3s_constexpr(&target_seed[0], target_seed.size());
    target_seed[seed_size + 1] = 1;
    const auto hash_lo = blake3::blake3s_constexpr(&target_seed[0], target_seed.size());
    // custom serialize methods as common/serialize.hpp is not constexpr!
    const auto read_uint256 = [](const uint8_t* in) {
        const auto read_limb = [](const uint8_t* in, uint64_t& out) {
            for (size_t i = 0; i < 8; ++i) {
                out += static_cast<uint64_t>(in[i]) << ((7 - i) * 8);
            }
        };
        uint256_t out = 0;
        read_limb(&in[0], out.data[3]);
        read_limb(&in[8], out.data[2]);
        read_limb(&in[16], out.data[1]);
        read_limb(&in[24], out.data[0]);
        return out;
    };
    // interpret 64 byte hash output as a uint512_t, reduce to Fq element
    //(512 bits of entropy ensures result is not biased as 512 >> Fq::modulus.get_msb())
    Fq x(uint512_t(read_uint256(&hash_lo[0]), read_uint256(&hash_hi[0])));
    bool sign_bit = hash_hi[0] > 127;
    std::optional<affine_element> result = derive_from_x_coordinate(x, sign_bit);
    if (result.has_value()) {
        return result.value();
    }
    return hash_to_curve(seed, attempt_count + 1);
}

template <typename Fq, typename Fr, typename T>
affine_element<Fq, Fr, T> affine_element<Fq, Fr, T>::random_element(numeric::random::Engine* engine) noexcept
{
    if (engine == nullptr) {
        engine = &numeric::random::get_engine();
    }

    Fq x;
    Fq y;
    while (true) {
        // Sample a random x-coordinate and check if it satisfies curve equation.
        x = Fq::random_element(engine);
        // Negate the y-coordinate based on a randomly sampled bit.
        bool sign_bit = (engine->get_random_uint8() & 1) != 0;

        std::optional<affine_element> result = derive_from_x_coordinate(x, sign_bit);

        if (result.has_value()) {
            return result.value();
        }
    }
    throw_or_abort("affine_element::random_element error");
    return affine_element<Fq, Fr, T>(x, y);
}

} // namespace bb::group_elements
