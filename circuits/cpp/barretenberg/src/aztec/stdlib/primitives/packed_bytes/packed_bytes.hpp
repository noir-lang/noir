#pragma once
#include "../composers/composers_fwd.hpp"
#include "../field/field.hpp"
#include "../bool/bool.hpp"
#include "../byte_array/byte_array.hpp"

namespace plonk {
namespace stdlib {

template <typename Composer> class packed_bytes {
    typedef field_t<Composer> field_t;
    typedef bool_t<Composer> bool_t;
    typedef byte_array<Composer> byte_array;

  public:
    packed_bytes(Composer* parent_context, size_t const num_bytes = 0);
    packed_bytes(const std::vector<field_t>& input, const size_t bytes_per_input = BYTES_PER_ELEMENT);
    packed_bytes(Composer* parent_context, const std::vector<uint8_t>& input);
    packed_bytes(Composer* parent_context, const std::string& input);
    packed_bytes(const byte_array& input);

    packed_bytes(const packed_bytes& other);
    packed_bytes(packed_bytes&& other);

    packed_bytes& operator=(const packed_bytes& other);
    packed_bytes& operator=(packed_bytes&& other);

    operator byte_array() const;

    std::vector<field_t> to_unverified_byte_slices(const size_t bytes_per_slice) const;
    std::vector<field_t> get_limbs() const { return limbs; }

    void append(const field_t& to_append, const size_t bytes_to_append);

    size_t size() const { return num_bytes; }

    Composer* get_context() const { return context; }

    std::string get_value() const;

  private:
    static constexpr uint64_t BYTES_PER_ELEMENT = 16;
    Composer* context;
    size_t num_bytes;
    std::vector<field_t> limbs;
};

template <typename Composer> inline std::ostream& operator<<(std::ostream& os, packed_bytes<Composer> const& arr)
{
    std::ios_base::fmtflags f(os.flags());
    os << "[" << std::hex << std::setfill('0');
    for (auto byte : arr.get_value()) {
        os << ' ' << std::setw(2) << +(unsigned char)byte;
    }
    os << " ]";
    os.flags(f);
    return os;
}

EXTERN_STDLIB_TYPE(packed_bytes);

} // namespace stdlib
} // namespace plonk