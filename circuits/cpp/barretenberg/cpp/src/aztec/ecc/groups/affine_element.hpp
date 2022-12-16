#pragma once
#include <numeric/uint256/uint256.hpp>
#include <vector>
#include <type_traits>
#include <ecc/curves/bn254/fq2.hpp>

namespace barretenberg {
namespace group_elements {
template <typename Fq, typename Fr, typename Params> class alignas(64) affine_element {
  public:
    affine_element() noexcept {}

    constexpr affine_element(const Fq& a, const Fq& b) noexcept;

    constexpr affine_element(const affine_element& other) noexcept;

    constexpr affine_element(affine_element&& other) noexcept;

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

    constexpr affine_element& operator=(const affine_element& other) noexcept;

    constexpr affine_element& operator=(affine_element&& other) noexcept;

    template <typename BaseField = Fq,
              typename CompileTimeEnabled = std::enable_if_t<(BaseField::modulus >> 255) == uint256_t(0), void>>
    constexpr uint256_t compress() const noexcept;

    constexpr affine_element set_infinity() const noexcept;
    constexpr void self_set_infinity() noexcept;

    constexpr bool is_point_at_infinity() const noexcept;

    constexpr bool on_curve() const noexcept;

    /**
     * @brief Hash a seed value to curve.
     *
     * @return A point on the curve corresponding to the given seed
     */
    static affine_element hash_to_curve(const uint64_t seed) noexcept;

    constexpr bool operator==(const affine_element& other) const noexcept;

    constexpr affine_element operator-() const noexcept { return { x, -y }; }

    constexpr bool operator>(const affine_element& other) const noexcept;
    constexpr bool operator<(const affine_element& other) const noexcept { return (other > *this); }

    /**
     * @brief Serialize the point to the given buffer
     *
     * @details We support serializing the point at infinity for curves defined over a barretenberg::field (i.e., a
     * native field of prime order) and for points of barretenberg::g2.
     *
     * @warning This will need to be updated if we serialize points over composite-order fields other than fq2!
     *
     */
    static void serialize_to_buffer(const affine_element& value, uint8_t* buffer)
    {
        if (value.is_point_at_infinity()) {
            if constexpr (Fq::modulus.get_msb() == 255) {
                write(buffer, uint256_t(0));
                write(buffer, Fq::modulus);
            } else {
                write(buffer, uint256_t(0));
                write(buffer, uint256_t(1) << 255);
            }
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
     * @details We support serializing the point at infinity for curves defined over a barretenberg::field (i.e., a
     * native field of prime order) and for points of barretenberg::g2.
     *
     * @warning This will need to be updated if we serialize points over composite-order fields other than fq2!
     */
    static affine_element serialize_from_buffer(uint8_t* buffer)
    {
        affine_element result;

        // need to read a raw uint256_t to avoid reductions so we can check whether the point is the point at infinity
        uint256_t raw_x = from_buffer<uint256_t>(buffer + sizeof(Fq));

        if constexpr (Fq::modulus.get_msb() == 255) {
            if (raw_x == Fq::modulus) {
                result.y = Fq::zero();
                result.x.data[0] = raw_x.data[0];
                result.x.data[1] = raw_x.data[1];
                result.x.data[2] = raw_x.data[2];
                result.x.data[3] = raw_x.data[3];
            } else {
                result.y = Fq::serialize_from_buffer(buffer);
                result.x = Fq(raw_x);
            }
        } else {
            if (raw_x.get_msb() == 255) {
                result.y = Fq::zero();
                result.x = Fq::zero();
                result.self_set_infinity();
            } else {
                // conditional here to avoid reading the same data twice in case of a field of prime order
                if constexpr (std::is_same<Fq, fq2>::value) {
                    result.y = Fq::serialize_from_buffer(buffer);
                    result.x = Fq::serialize_from_buffer(buffer + sizeof(Fq));
                } else {
                    result.y = Fq::serialize_from_buffer(buffer);
                    result.x = Fq(raw_x);
                }
            }
        }
        return result;
    }

    /**
     * @brief Serialize the point to a byte vector
     *
     * @return Vector with serialized representation of the point
     */
    inline std::vector<uint8_t> to_buffer() const
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
};

template <typename B, typename Fq, typename Fr, typename Params> void read(B& it, affine_element<Fq, Fr, Params>& value)
{
    read(it, value.x);
    read(it, value.y);
}

template <typename B, typename Fq, typename Fr, typename Params>
void write(B& buf, affine_element<Fq, Fr, Params> const& value)
{
    write(buf, value.x);
    write(buf, value.y);
}

} // namespace group_elements
} // namespace barretenberg

#include "./affine_element_impl.hpp"