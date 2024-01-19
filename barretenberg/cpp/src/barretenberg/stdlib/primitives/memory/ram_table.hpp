#pragma once
#include "../circuit_builders/circuit_builders_fwd.hpp"
#include "../field/field.hpp"

namespace bb::plonk {
namespace stdlib {

// A runtime-defined read-only memory table. Table entries must be initialized in the constructor.
// N.B. Only works with the UltraPlonkBuilder at the moment!
template <typename Builder> class ram_table {
  private:
    typedef field_t<Builder> field_pt;

  public:
    ram_table() {}
    ram_table(Builder* builder, const size_t table_size);
    ram_table(const std::vector<field_pt>& table_entries);
    ram_table(const ram_table& other);
    ram_table(ram_table&& other);

    void initialize_table() const;

    ram_table& operator=(const ram_table& other);
    ram_table& operator=(ram_table&& other);

    field_pt read(const field_pt& index) const;

    void write(const field_pt& index, const field_pt& value);

    size_t size() const { return _length; }

    Builder* get_context() const { return _context; }

    bool check_indices_initialized() const
    {
        if (_all_entries_written_to_with_constant_index) {
            return true;
        }
        if (_length == 0) {
            return false;
        }
        bool init = true;
        for (auto i : _index_initialized) {
            init = init && i;
        }
        _all_entries_written_to_with_constant_index = init;
        return _all_entries_written_to_with_constant_index;
    }

  private:
    std::vector<field_pt> _raw_entries;
    mutable std::vector<bool> _index_initialized;
    size_t _length = 0;
    mutable size_t _ram_id = 0; // Builder identifier for this ROM table
    mutable bool _ram_table_generated_in_builder = false;
    mutable bool _all_entries_written_to_with_constant_index = false;
    mutable Builder* _context = nullptr;
};
} // namespace stdlib
} // namespace bb::plonk