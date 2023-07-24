#pragma once
#include "../circuit_builders/circuit_builders_fwd.hpp"
#include "ram_table.hpp"
namespace proof_system::plonk {
namespace stdlib {

/**
 * @brief A dynamic array of field elements
 *
 * @tparam Composer (must support plookup)
 */
template <typename Composer> class DynamicArray {
  private:
    typedef field_t<Composer> field_pt;
    typedef bool_t<Composer> bool_pt;
    typedef witness_t<Composer> witness_pt;

  public:
    DynamicArray(Composer* composer, const size_t maximum_size);

    DynamicArray(const DynamicArray& other);
    DynamicArray(DynamicArray&& other);

    DynamicArray& operator=(const DynamicArray& other);
    DynamicArray& operator=(DynamicArray&& other);

    void resize(const field_pt& new_length, const field_pt default_value = 0);

    field_pt read(const field_pt& index) const;
    void write(const field_pt& index, const field_pt& value);

    void push(const field_pt& index);
    void pop();

    void conditional_push(const bool_pt& predicate, const field_pt& index);
    void conditional_pop(const bool_pt& predicate);

    field_pt size() const { return _length; }
    size_t native_size() const { return static_cast<size_t>(static_cast<uint256_t>(_length.get_value())); }
    size_t max_size() const { return _max_size; }

    Composer* get_context() const { return _context; }

  private:
    Composer* _context = nullptr;
    size_t _max_size;
    field_pt _length = 0;
    mutable ram_table<Composer> _inner_table;
};

EXTERN_STDLIB_ULTRA_TYPE(DynamicArray);

} // namespace stdlib
} // namespace proof_system::plonk