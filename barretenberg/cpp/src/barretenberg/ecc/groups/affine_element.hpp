#pragma once
#include "barretenberg/ecc/curves/bn254/fq2.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstring>
#include <type_traits>
#include <vector>

namespace bb::group_elements {
template <typename T>
concept SupportsHashToCurve = T::can_hash_to_curve;
template <typename Fq_, typename Fr_, typename Params> class alignas(64) affine_element {
  public:
    using Fq = Fq_;
    using Fr = Fr_;

    using in_buf = const uint8_t*;
    using vec_in_buf = const uint8_t*;
    using out_buf = uint8_t*;
    using vec_out_buf = uint8_t**;

    affine_element() noexcept = default;
    ~affine_element() noexcept = default;

    constexpr affine_element(const Fq& x, const Fq& y) noexcept;

    constexpr affine_element(const affine_element& other) noexcept = default;

    constexpr affine_element(affine_element&& other) noexcept = default;

    static constexpr affine_element one() noexcept { return { Params::one_x, Params::one_y }; };

    /**
     * @brief Reconstruct a point in affine coordinates from compressed form.
     * @details #LARGE_MODULUS_AFFINE_POINT_COMPRESSION Point compression is only implemented for curves of a prime
     * field F_p with p using < 256 bits.  One possiblity for extending to a 256-bit prime field:
     * https://patents.google.com/patent/US6252960B1/en.
     *
     * @param compressed compressed point
     * @return constexpr affine_element
     */
    template <typename BaseField = Fq,
              typename CompileTimeEnabled = std::enable_if_t<(BaseField::modulus >> 255) == uint256_t(0), void>>
    static constexpr affine_element from_compressed(const uint256_t& compressed) noexcept;

    /**
     * @brief Reconstruct a point in affine coordinates from compressed form.
     * @details #LARGE_MODULUS_AFFINE_POINT_COMPRESSION Point compression is implemented for curves of a prime
     * field F_p with p being 256 bits.
     * TODO(Suyash): Check with kesha if this is correct.
     *
     * @param compressed compressed point
     * @return constexpr affine_element
     */
    template <typename BaseField = Fq,
              typename CompileTimeEnabled = std::enable_if_t<(BaseField::modulus >> 255) == uint256_t(1), void>>
    static constexpr std::array<affine_element, 2> from_compressed_unsafe(const uint256_t& compressed) noexcept;

    constexpr affine_element& operator=(const affine_element& other) noexcept = default;

    constexpr affine_element& operator=(affine_element&& other) noexcept = default;

    constexpr affine_element operator+(const affine_element& other) const noexcept;

    template <typename BaseField = Fq,
              typename CompileTimeEnabled = std::enable_if_t<(BaseField::modulus >> 255) == uint256_t(0), void>>
    [[nodiscard]] constexpr uint256_t compress() const noexcept;

    static affine_element infinity();
    constexpr affine_element set_infinity() const noexcept;
    constexpr void self_set_infinity() noexcept;

    [[nodiscard]] constexpr bool is_point_at_infinity() const noexcept;

    [[nodiscard]] constexpr bool on_curve() const noexcept;

    static constexpr std::optional<affine_element> derive_from_x_coordinate(const Fq& x, bool sign_bit) noexcept;

    /**
     * @brief Samples a random point on the curve.
     *
     * @return A randomly chosen point on the curve
     */
    static affine_element random_element(numeric::RNG* engine = nullptr) noexcept;
    static constexpr affine_element hash_to_curve(const std::vector<uint8_t>& seed, uint8_t attempt_count = 0) noexcept
        requires SupportsHashToCurve<Params>;

    constexpr bool operator==(const affine_element& other) const noexcept;

    constexpr affine_element operator-() const noexcept { return { x, -y }; }

    constexpr bool operator>(const affine_element& other) const noexcept;
    constexpr bool operator<(const affine_element& other) const noexcept { return (other > *this); }

    /**
     * @brief Serialize the point to the given buffer
     *
     * @details We support serializing the point at infinity for curves defined over a bb::field (i.e., a
     * native field of prime order) and for points of bb::g2.
     *
     * @warning This will need to be updated if we serialize points over composite-order fields other than fq2!
     *
     */
    static void serialize_to_buffer(const affine_element& value, uint8_t* buffer)
    {
        if (value.is_point_at_infinity()) {
            // if we are infinity, just set all buffer bits to 1
            // we only need this case because the below gets mangled converting from montgomery for infinity points
            memset(buffer, 255, sizeof(Fq) * 2);
        } else {
            Fq::serialize_to_buffer(value.y, buffer);
            Fq::serialize_to_buffer(value.x, buffer + sizeof(Fq));
        }
    }

    /**
     * @brief Restore point from a buffer
     *
     * @param buffer Buffer from which we deserialize the point
     *
     * @return Deserialized point
     *
     * @details We support serializing the point at infinity for curves defined over a bb::field (i.e., a
     * native field of prime order) and for points of bb::g2.
     *
     * @warning This will need to be updated if we serialize points over composite-order fields other than fq2!
     */
    static affine_element serialize_from_buffer(uint8_t* buffer)
    {
        // Does the buffer consist entirely of set bits? If so, we have a point at infinity
        // Note that if it isn't, this loop should end early.
        // We only need this case because the below gets mangled converting to montgomery for infinity points
        bool is_point_at_infinity =
            std::all_of(buffer, buffer + sizeof(Fq) * 2, [](uint8_t val) { return val == 255; });
        if (is_point_at_infinity) {
            return affine_element::infinity();
        }
        affine_element result;
        result.y = Fq::serialize_from_buffer(buffer);
        result.x = Fq::serialize_from_buffer(buffer + sizeof(Fq));
        return result;
    }

    /**
     * @brief Serialize the point to a byte vector
     *
     * @return Vector with serialized representation of the point
     */
    [[nodiscard]] inline std::vector<uint8_t> to_buffer() const
    {
        std::vector<uint8_t> buffer(sizeof(affine_element));
        affine_element::serialize_to_buffer(*this, &buffer[0]);
        return buffer;
    }

    friend std::ostream& operator<<(std::ostream& os, const affine_element& a)
    {
        os << "{ " << a.x << ", " << a.y << " }";
        return os;
    }
    Fq x;
    Fq y;

    // for serialization: update with new fields
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/908) point at inifinty isn't handled
    MSGPACK_FIELDS(x, y);
};
} // namespace bb::group_elements

#include "./affine_element_impl.hpp"
