#pragma once
#include <numeric/uint256/uint256.hpp>
#include <vector>

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
     *
     * @tparam BaseField Coordinate field class
     * @tparam CompileTimeEnabled Checks that the modulus of BaseField is < 2**255, otherwise disables the function
     *
     * @param compressed Compressed point
     */
    template <typename BaseField = Fq,
              typename CompileTimeEnabled = std::enable_if_t<(BaseField::modulus >> 255) == uint256_t(0), void>>
    static constexpr std::pair<bool, affine_element> deserialize(const uint256_t& compressed) noexcept;

    constexpr affine_element& operator=(const affine_element& other) noexcept;

    constexpr affine_element& operator=(affine_element&& other) noexcept;

    template <typename BaseField = Fq,
              typename CompileTimeEnabled = std::enable_if_t<(BaseField::modulus >> 255) == uint256_t(0), void>>
    explicit constexpr operator uint256_t() const noexcept;

    constexpr affine_element set_infinity() const noexcept;
    constexpr void self_set_infinity() noexcept;

    constexpr bool is_point_at_infinity() const noexcept;

    constexpr bool on_curve() const noexcept;

    /**
     * @brief Hash a seed value to curve.
     *
     * @tparam BaseField Coordinate field
     * @tparam CompileTimeEnabled Checks that the modulus of BaseField is < 2**255, otherwise disables the function
     *
     * @return <true,A point on the curve corresponding to the given seed> if the seed lands on a point <false,
     * affine_element(0,0)> if not.
     */
    template <typename BaseField = Fq,
              typename CompileTimeEnabled = std::enable_if_t<(BaseField::modulus >> 255) == uint256_t(0), void>>
    static std::pair<bool, affine_element> hash_to_curve(const uint64_t seed) noexcept;

    constexpr bool operator==(const affine_element& other) const noexcept;

    constexpr affine_element operator-() const noexcept { return { x, -y }; }

    /**
     * @brief Serialize the point to the given buffer
     *
     * @tparam BaseField Coordinate field
     * @tparam CompileTimeEnabled Checks that the modulus of BaseField is < 2**255, otherwise disables the function
     *
     */
    template <typename BaseField = Fq,
              typename CompileTimeEnabled = std::enable_if_t<(BaseField::modulus >> 255) == uint256_t(0), void>>
    static void serialize_to_buffer(const affine_element& value, uint8_t* buffer)
    {
        Fq::serialize_to_buffer(value.y, buffer);
        Fq::serialize_to_buffer(value.x, buffer + sizeof(Fq));
        if (value.is_point_at_infinity()) {
            buffer[0] = buffer[0] | (1 << 7);
        }
    }
    /**
     * @brief Restore point from a buffer
     *
     * @tparam BaseField Coordinate field
     * @tparam CompileTimeEnabled Checks that the modulus of BaseField is < 2**255, otherwise disables the function
     *
     * @param buffer Buffer from which we deserialize the point
     *
     * @return Deserialized point
     */
    template <typename BaseField = Fq,
              typename CompileTimeEnabled = std::enable_if_t<(BaseField::modulus >> 255) == uint256_t(0), void>>
    static affine_element serialize_from_buffer(uint8_t* buffer)
    {
        affine_element result;
        result.y = Fq::serialize_from_buffer(buffer);
        result.x = Fq::serialize_from_buffer(buffer + sizeof(Fq));
        if (((buffer[0] >> 7) & 1) == 1) {
            result.self_set_infinity();
        }
        return result;
    }
    /**
     * @brief Serialize the point to a byte vector
     *
     * @tparam BaseField Coordinate field
     * @tparam CompileTimeEnabled Checks that the modulus of BaseField is < 2**255, otherwise disables the function
     *
     * @return Vector with serialized representation of the point
     */
    template <typename BaseField = Fq,
              typename CompileTimeEnabled = std::enable_if_t<(BaseField::modulus >> 255) == uint256_t(0), void>>
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