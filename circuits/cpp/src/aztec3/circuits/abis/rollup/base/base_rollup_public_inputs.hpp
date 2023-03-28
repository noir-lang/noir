#pragma once
#include <barretenberg/stdlib/recursion/aggregation_state/aggregation_state.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include "../../append_only_tree_snapshot.hpp"
#include "../../append_only_tree_snapshot.hpp"
#include "../constant_rollup_data.hpp"

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct BaseRollupPublicInputs {
    typedef typename NCT::fr fr;
    typedef typename NCT::AggregationObject AggregationObject;

    AggregationObject end_aggregation_object;
    ConstantRollupData<NCT> constants;

    AppendOnlyTreeSnapshot<NCT> start_private_data_tree_snapshot;
    AppendOnlyTreeSnapshot<NCT> end_private_data_tree_snapshot;

    AppendOnlyTreeSnapshot<NCT> start_nullifier_tree_snapshot;
    AppendOnlyTreeSnapshot<NCT> end_nullifier_tree_snapshot;

    AppendOnlyTreeSnapshot<NCT> start_contract_tree_snapshot;
    AppendOnlyTreeSnapshot<NCT> end_contract_tree_snapshot;

    // Hashes (probably sha256) to make public inputs constant-sized (to then be unpacked on-chain)
    // UPDATE we should instead just hash all of the below into a single value. See big diagram of sha256 hashing
    // bottom-right of here.
    // TODO I've put `fr`, but these hash values' types might need to be two fields if we want all 256-bits, for
    // security purposes.
    std::array<fr, 2> calldata_hash;

    bool operator==(BaseRollupPublicInputs<NCT> const&) const = default;
};

template <typename NCT> void read(uint8_t const*& it, BaseRollupPublicInputs<NCT>& obj)
{
    using serialize::read;

    read(it, obj.end_aggregation_object);
    read(it, obj.constants);
    read(it, obj.start_private_data_tree_snapshot);
    read(it, obj.end_private_data_tree_snapshot);
    read(it, obj.start_nullifier_tree_snapshot);
    read(it, obj.end_nullifier_tree_snapshot);
    read(it, obj.start_contract_tree_snapshot);
    read(it, obj.end_contract_tree_snapshot);
    read(it, obj.calldata_hash);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, BaseRollupPublicInputs<NCT> const& obj)
{
    using serialize::write;

    write(buf, obj.end_aggregation_object);
    write(buf, obj.constants);
    write(buf, obj.start_private_data_tree_snapshot);
    write(buf, obj.end_private_data_tree_snapshot);
    write(buf, obj.start_nullifier_tree_snapshot);
    write(buf, obj.end_nullifier_tree_snapshot);
    write(buf, obj.start_contract_tree_snapshot);
    write(buf, obj.end_contract_tree_snapshot);
    write(buf, obj.calldata_hash);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, BaseRollupPublicInputs<NCT> const& obj)
{
    return os << "end_aggregation_object:\n"
              << obj.end_aggregation_object
              << "\n"
                 "constants:\n"
              << obj.constants
              << "\n"
                 "start_private_data_tree_snapshot:\n"
              << obj.start_private_data_tree_snapshot
              << "\n"
                 "end_private_data_tree_snapshot:\n"
              << obj.start_private_data_tree_snapshot
              << "\n"
                 "start_nullifier_tree_snapshot:\n"
              << obj.start_nullifier_tree_snapshot
              << "\n"
                 "end_nullifier_tree_snapshots:\n"
              << obj.end_nullifier_tree_snapshot
              << "\n"
                 "start_contract_tree_snapshot:\n"
              << obj.start_contract_tree_snapshot
              << "\n"
                 "end_contract_tree_snapshot:\n"
              << obj.end_contract_tree_snapshot
              << "\n"
                 "calldata_hash: "
              << obj.calldata_hash << "\n";
}

} // namespace aztec3::circuits::abis