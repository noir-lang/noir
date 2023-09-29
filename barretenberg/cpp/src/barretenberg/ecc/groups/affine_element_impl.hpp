#pragma once
#include "./element.hpp"
#include "barretenberg/crypto/keccak/keccak.hpp"
#include "barretenberg/crypto/sha256/sha256.hpp"

namespace barretenberg::group_elements {
template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T>::affine_element(const Fq& a, const Fq& b) noexcept
    : x(a)
    , y(b)
{}

template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T>::affine_element(const affine_element& other) noexcept
    : x(other.x)
    , y(other.y)
{}

template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T>::affine_element(affine_element&& other) noexcept
    : x(other.x)
    , y(other.y)
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
constexpr affine_element<Fq, Fr, T>& affine_element<Fq, Fr, T>::operator=(const affine_element& other) noexcept
{
    if (this == &other) {
        return *this;
    }
    x = other.x;
    y = other.y;
    return *this;
}

template <class Fq, class Fr, class T>
constexpr affine_element<Fq, Fr, T>& affine_element<Fq, Fr, T>::operator=(affine_element&& other) noexcept
{
    x = other.x;
    y = other.y;
    return *this;
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
        // This is for determinism; a different sqrt algorithm could give -y instead of y and so this parity check
        // allows all algorithms to get the "same" y
        if (uint256_t(y).get_bit(0) != sign_bit) {
            y = -y;
        }
        return affine_element(x, y);
    }
    return std::nullopt;
}

template <class Fq, class Fr, class T>
template <typename>
affine_element<Fq, Fr, T> affine_element<Fq, Fr, T>::hash_to_curve(uint64_t seed) noexcept
{
    static_assert(static_cast<bool>(T::can_hash_to_curve));

    Fq input(seed, 0, 0, 0);
    keccak256 c = hash_field_element(&input.data[0]);
    uint256_t hash{ c.word64s[0], c.word64s[1], c.word64s[2], c.word64s[3] };

    uint256_t x_coordinate = hash;

    if constexpr (Fq::modulus.data[3] < 0x8000000000000000ULL) {
        x_coordinate.data[3] = x_coordinate.data[3] & (~0x8000000000000000ULL);
    }

    bool y_bit = hash.get_bit(255);

    std::optional<affine_element> result = derive_from_x_coordinate(x_coordinate, y_bit);

    if (result.has_value()) {
        return result.value();
    }
    return affine_element(0, 0);
}

template <class Fq, class Fr, class T>
affine_element<Fq, Fr, T> affine_element<Fq, Fr, T>::hash_to_curve(const std::vector<uint8_t>& seed,
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
    target_seed.back() = 0;
    const auto hash_hi = sha256::sha256(target_seed);
    target_seed.back() = 1;
    const auto hash_lo = sha256::sha256(target_seed);
    // custom serialize methods as common/serialize.hpp is not constexpr
    // (next PR will make this method constexpr)
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

} // namespace barretenberg::group_elements
