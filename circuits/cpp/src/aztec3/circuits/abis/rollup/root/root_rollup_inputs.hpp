

#pragma once
#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include "aztec3/circuits/abis/rollup/base/previous_rollup_data.hpp"
#include "aztec3/constants.hpp"
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <ostream>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct RootRollupInputs {
    typedef typename NCT::fr fr;

    // All below are shared between the base and merge rollups
    std::array<PreviousRollupData<NCT>, 2> previous_rollup_data;

    std::array<fr, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT> new_historic_private_data_tree_root_sibling_path;
    std::array<fr, CONTRACT_TREE_ROOTS_TREE_HEIGHT> new_historic_contract_tree_root_sibling_path;

    bool operator==(RootRollupInputs<NCT> const&) const = default;
};

template <typename NCT> void read(uint8_t const*& it, RootRollupInputs<NCT>& obj)
{
    using serialize::read;

    read(it, obj.previous_rollup_data);
    read(it, obj.new_historic_private_data_tree_root_sibling_path);
    read(it, obj.new_historic_contract_tree_root_sibling_path);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, RootRollupInputs<NCT> const& obj)
{
    using serialize::write;

    write(buf, obj.previous_rollup_data);
    write(buf, obj.new_historic_private_data_tree_root_sibling_path);
    write(buf, obj.new_historic_contract_tree_root_sibling_path);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, RootRollupInputs<NCT> const& obj)
{
    return os << "previous_rollup_data: " << obj.previous_rollup_data << "\n"
              << "new_historic_private_data_tree_roots: " << obj.new_historic_private_data_tree_root_sibling_path
              << "\n"
              << "new_historic_contract_tree_roots: " << obj.new_historic_contract_tree_root_sibling_path << "\n";
}

} // namespace aztec3::circuits::abis