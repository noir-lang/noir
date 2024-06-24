#include "barretenberg/vm/avm_trace/fixed_powers.hpp"

#include <cstdint>

#include "barretenberg/numeric/uint256/uint256.hpp"

namespace bb::avm_trace {

FixedPowersTable::FixedPowersTable()
{
    for (uint64_t i = 0; i < 256; i++) {
        table_rows.push_back(PowersRow{
            .powers_power_of_2 = FF(uint256_t(1) << uint256_t(i)),
        });
    }
}

// Singleton.
const FixedPowersTable& FixedPowersTable::get()
{
    static FixedPowersTable table;
    return table;
}

} // namespace bb::avm_trace