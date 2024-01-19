#pragma once
#include "../bool/bool.hpp"
#include "../circuit_builders/circuit_builders_fwd.hpp"
#include "../field/field.hpp"
#include "../safe_uint/safe_uint.hpp"
namespace bb::plonk {
namespace stdlib {

template <typename Builder> class byte_array {
  public:
    typedef std::vector<field_t<Builder>> bytes_t;

    byte_array(Builder* parent_context = nullptr);
    byte_array(Builder* parent_context, size_t const n);
    byte_array(Builder* parent_context, std::string const& input);
    byte_array(Builder* parent_context, std::vector<uint8_t> const& input);
    byte_array(Builder* parent_context, bytes_t const& input);
    byte_array(Builder* parent_context, bytes_t&& input);
    byte_array(const field_t<Builder>& input, const size_t num_bytes = 32);
    byte_array(const safe_uint_t<Builder>& input, const size_t num_bytes = 32);

    template <typename ItBegin, typename ItEnd>
    byte_array(Builder* parent_context, ItBegin const& begin, ItEnd const& end)
        : context(parent_context)
        , values(begin, end)
    {
        for (auto& val : values) {
            val = val.normalize();
        }
    }

    byte_array(const byte_array& other);
    byte_array(byte_array&& other);

    byte_array& operator=(const byte_array& other);
    byte_array& operator=(byte_array&& other);

    explicit operator field_t<Builder>() const;

    field_t<Builder> operator[](const size_t index) const
    {
        assert(values.size() > 0);
        return values[index];
    }

    byte_array& write(byte_array const& other);
    byte_array& write_at(byte_array const& other, size_t index);

    byte_array slice(size_t offset) const;
    byte_array slice(size_t offset, size_t length) const;
    byte_array reverse() const;

    size_t size() const { return values.size(); }

    bytes_t const& bytes() const { return values; }

    bool_t<Builder> get_bit(size_t index) const;

    void set_bit(size_t index, bool_t<Builder> const& value);

    void set_byte(size_t index, const field_t<Builder>& byte_val)
    {
        ASSERT(index < values.size());
        values[index] = byte_val;
    }

    void set_context(Builder* ctx)
    {
        ASSERT(context == nullptr);
        context = ctx;
    }

    Builder* get_context() const { return context; }

    std::vector<uint8_t> get_value() const;

    std::string get_string() const;

  private:
    Builder* context;
    bytes_t values;

    struct byte_slice {
        field_t<Builder> low;
        field_t<Builder> high;
        bool_t<Builder> bit;
    };
    byte_slice split_byte(const size_t bit_index) const;
};

template <typename Builder> inline std::ostream& operator<<(std::ostream& os, byte_array<Builder> const& arr)
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
} // namespace stdlib
} // namespace bb::plonk
