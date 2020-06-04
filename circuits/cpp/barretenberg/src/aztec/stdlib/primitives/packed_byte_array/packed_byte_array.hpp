#pragma once
#include "../composers/composers_fwd.hpp"
#include "../field/field.hpp"
#include "../bool/bool.hpp"
#include "../byte_array/byte_array.hpp"

namespace plonk {
namespace stdlib {

template <typename Composer> class packed_byte_array {
  private:
    typedef field_t<Composer> field_pt;
    typedef bool_t<Composer> bool_pt;
    typedef byte_array<Composer> byte_array_pt;

  public:
    packed_byte_array(Composer* parent_context, size_t const num_bytes = 0);
    packed_byte_array(const std::vector<field_pt>& input, const size_t bytes_per_input = BYTES_PER_ELEMENT);
    packed_byte_array(Composer* parent_context, const std::vector<uint8_t>& input);
    packed_byte_array(Composer* parent_context, const std::string& input);
    packed_byte_array(const byte_array_pt& input);

    packed_byte_array(const packed_byte_array& other);
    packed_byte_array(packed_byte_array&& other);

    packed_byte_array& operator=(const packed_byte_array& other);
    packed_byte_array& operator=(packed_byte_array&& other);

    operator byte_array_pt() const;

    std::vector<field_pt> to_unverified_byte_slices(const size_t bytes_per_slice) const;
    std::vector<field_pt> get_limbs() const { return limbs; }

    void append(const field_pt& to_append, const size_t bytes_to_append);

    size_t size() const { return num_bytes; }

    Composer* get_context() const { return context; }

    std::string get_value() const;

  private:
    static constexpr uint64_t BYTES_PER_ELEMENT = 16;
    Composer* context;
    size_t num_bytes;
    std::vector<field_pt> limbs;
};

template <typename Composer> inline std::ostream& operator<<(std::ostream& os, packed_byte_array<Composer> const& arr)
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

EXTERN_STDLIB_TYPE(packed_byte_array);

} // namespace stdlib
} // namespace plonk