#pragma once
#include "../bool/bool.hpp"
#include "../composers/composers_fwd.hpp"

namespace plonk {
namespace stdlib {

template <typename ComposerContext> class byte_array {
  public:
    typedef std::vector<bool_t<ComposerContext>> bits_t;

    byte_array(ComposerContext* parent_context);
    byte_array(ComposerContext* parent_context, size_t const n);
    byte_array(ComposerContext* parent_context, std::string const& input);
    byte_array(ComposerContext* parent_context, std::vector<uint8_t> const& input);
    byte_array(ComposerContext* parent_context, bits_t const& input);
    byte_array(ComposerContext* parent_context, bits_t&& input);

    template <typename ItBegin, typename ItEnd>
    byte_array(ComposerContext* parent_context, ItBegin const& begin, ItEnd const& end)
        : context(parent_context)
        , values(begin, end)
    {}

    byte_array(const byte_array& other);
    byte_array(byte_array&& other);

    byte_array& operator=(const byte_array& other);
    byte_array& operator=(byte_array&& other);

    byte_array& write(byte_array const& other);

    byte_array slice(size_t offset) const;
    byte_array slice(size_t offset, size_t length) const;
    byte_array reverse() const;

    size_t size() const { return values.size() / 8; }

    bits_t const& bits() const { return values; }

    bool_t<ComposerContext> const& get_bit(size_t index) const { return values[values.size() - index - 1]; }

    void set_bit(size_t index, bool_t<ComposerContext> const& value) { values[index] = value; }

    ComposerContext* get_context() const { return context; }

    std::string get_value() const;

  private:
    ComposerContext* context;
    bits_t values;
};

template <typename ComposerContext>
inline std::ostream& operator<<(std::ostream& os, byte_array<ComposerContext> const& arr)
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

EXTERN_STDLIB_TYPE(byte_array);

} // namespace stdlib
} // namespace plonk