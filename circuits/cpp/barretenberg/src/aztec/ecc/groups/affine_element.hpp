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

    explicit constexpr affine_element(const uint256_t& compressed) noexcept;

    constexpr affine_element& operator=(const affine_element& other) noexcept;

    constexpr affine_element& operator=(affine_element&& other) noexcept;

    explicit constexpr operator uint256_t() const noexcept;

    constexpr affine_element set_infinity() const noexcept;
    constexpr void self_set_infinity() noexcept;

    constexpr bool is_point_at_infinity() const noexcept;

    constexpr bool on_curve() const noexcept;

    static affine_element hash_to_curve(const uint64_t seed) noexcept;

    constexpr bool operator==(const affine_element& other) const noexcept;

    constexpr affine_element operator-() const noexcept
    {
        return { x, -y };
    }

    static void serialize_to_buffer(const affine_element& value, uint8_t* buffer)
    {
        Fq::serialize_to_buffer(value.y, buffer);
        Fq::serialize_to_buffer(value.x, buffer + sizeof(Fq));
        if (!value.on_curve()) {
            buffer[0] = buffer[0] | (1 << 7);
        }
    }

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

template <typename Fq, typename Fr, typename Params>
void read(uint8_t*& it, affine_element<Fq, Fr, Params>& value) {
    value = affine_element<Fq, Fr, Params>::serialize_from_buffer(it);
    it += 64;
}

template <typename Fq, typename Fr, typename Params>
void write(std::vector<uint8_t>& buf, affine_element<Fq, Fr, Params> const& value) {
    buf.resize(buf.size() + 64);
    affine_element<Fq, Fr, Params>::serialize_to_buffer(value, &*buf.end() - 64);
}

} // namespace group_elements
} // namespace barretenberg

#include "./affine_element_impl.hpp"