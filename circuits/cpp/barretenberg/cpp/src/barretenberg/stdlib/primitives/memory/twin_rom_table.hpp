#pragma once
#include "../composers/composers_fwd.hpp"
#include "../field/field.hpp"

namespace plonk {
namespace stdlib {

// A runtime-defined read-only memory table. Table entries must be initialized in the constructor.
// Each entry contains a pair of values
// N.B. Only works with the UltraComposer at the moment!
template <typename Composer> class twin_rom_table {
  private:
    typedef field_t<Composer> field_pt;
    typedef std::array<field_pt, 2> field_pair_pt;

  public:
    twin_rom_table(){};
    twin_rom_table(const std::vector<field_pair_pt>& table_entries);
    twin_rom_table(const twin_rom_table& other);
    twin_rom_table(twin_rom_table&& other);

    void initialize_table() const;

    twin_rom_table& operator=(const twin_rom_table& other);
    twin_rom_table& operator=(twin_rom_table&& other);

    // read from table with a constant index value. Does not add any gates
    field_pair_pt operator[](const size_t index) const;

    // read from table with a witness index value. Adds 2 gates
    field_pair_pt operator[](const field_pt& index) const;

    size_t size() const { return length; }

    Composer* get_context() const { return context; }

  private:
    std::vector<field_pair_pt> raw_entries;
    mutable std::vector<field_pair_pt> entries;
    size_t length = 0;
    mutable size_t rom_id = 0; // Composer identifier for this ROM table
    mutable bool initialized = false;
    mutable Composer* context = nullptr;
};

EXTERN_STDLIB_ULTRA_TYPE(twin_rom_table);

} // namespace stdlib
} // namespace plonk