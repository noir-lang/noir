#include "barretenberg/vm/avm_trace/fixed_gas.hpp"

namespace bb::avm_trace {

FixedGasTable::FixedGasTable()
{
    for (int i = 0; i < static_cast<int>(OpCode::LAST_OPCODE_SENTINEL); i++) {
        table_rows.push_back(GasRow{
            .gas_da_gas_fixed_table = FF(2),
            .gas_l2_gas_fixed_table = FF(10),
            .gas_sel_gas_cost = FF(1),
        });
    }
}

// Singleton.
const FixedGasTable& FixedGasTable::get()
{
    static FixedGasTable table;
    return table;
}

} // namespace bb::avm_trace