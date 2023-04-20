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

const uint32_t BASE_ROLLUP_TYPE = 0;
const uint32_t MERGE_ROLLUP_TYPE = 1;

template <typename NCT> struct BaseOrMergeRollupPublicInputs {
    typedef typename NCT::fr fr;
    typedef typename NCT::AggregationObject AggregationObject;

    uint32_t rollup_type;
    // subtree  height is always 0 for base.
    // so that we always pass-in two base/merge circuits of the same height into the next level of recursion
    fr rollup_subtree_height;

    AggregationObject end_aggregation_object;
    ConstantRollupData<NCT> constants;

    AppendOnlyTreeSnapshot<NCT> start_private_data_tree_snapshot;
    AppendOnlyTreeSnapshot<NCT> end_private_data_tree_snapshot;

    AppendOnlyTreeSnapshot<NCT> start_nullifier_tree_snapshot;
    AppendOnlyTreeSnapshot<NCT> end_nullifier_tree_snapshot;

    AppendOnlyTreeSnapshot<NCT> start_contract_tree_snapshot;
    AppendOnlyTreeSnapshot<NCT> end_contract_tree_snapshot;

    AppendOnlyTreeSnapshot<NCT> start_public_data_tree_snapshot;
    AppendOnlyTreeSnapshot<NCT> end_public_data_tree_snapshot;

    // Hashes (probably sha256) to make public inputs constant-sized (to then be unpacked on-chain)
    // Needs to be two fields to accomodate all 256-bits of the hash
    std::array<fr, 2> calldata_hash;

    bool operator==(BaseOrMergeRollupPublicInputs<NCT> const&) const = default;
};

template <typename NCT> void read(uint8_t const*& it, BaseOrMergeRollupPublicInputs<NCT>& obj)
{
    using serialize::read;

    read(it, obj.rollup_type);
    read(it, obj.rollup_subtree_height);
    read(it, obj.end_aggregation_object);
    read(it, obj.constants);
    read(it, obj.start_private_data_tree_snapshot);
    read(it, obj.end_private_data_tree_snapshot);
    read(it, obj.start_nullifier_tree_snapshot);
    read(it, obj.end_nullifier_tree_snapshot);
    read(it, obj.start_contract_tree_snapshot);
    read(it, obj.end_contract_tree_snapshot);
    read(it, obj.start_public_data_tree_snapshot);
    read(it, obj.end_public_data_tree_snapshot);
    read(it, obj.calldata_hash);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, BaseOrMergeRollupPublicInputs<NCT> const& obj)
{
    using serialize::write;

    write(buf, obj.rollup_type);
    write(buf, obj.rollup_subtree_height);
    write(buf, obj.end_aggregation_object);
    write(buf, obj.constants);
    write(buf, obj.start_private_data_tree_snapshot);
    write(buf, obj.end_private_data_tree_snapshot);
    write(buf, obj.start_nullifier_tree_snapshot);
    write(buf, obj.end_nullifier_tree_snapshot);
    write(buf, obj.start_contract_tree_snapshot);
    write(buf, obj.end_contract_tree_snapshot);
    write(buf, obj.start_public_data_tree_snapshot);
    write(buf, obj.end_public_data_tree_snapshot);
    write(buf, obj.calldata_hash);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, BaseOrMergeRollupPublicInputs<NCT> const& obj)
{
    return os << "rollup_type:\n"
              << obj.rollup_type << "\n"
              << "rollup_subtree_height:\n"
              << obj.rollup_subtree_height << "\n"
              << "end_aggregation_object:\n"
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
                 "end_nullifier_tree_snapshot:\n"
              << obj.end_nullifier_tree_snapshot
              << "\n"
                 "start_contract_tree_snapshot:\n"
              << obj.start_contract_tree_snapshot
              << "\n"
                 "end_contract_tree_snapshot:\n"
              << obj.end_contract_tree_snapshot
              << "\n"
                 "start_public_data_tree_snapshot:\n"
              << obj.start_public_data_tree_snapshot
              << "\n"
                 "end_public_data_tree_snapshot:\n"
              << obj.end_public_data_tree_snapshot
              << "\n"
                 "calldata_hash: "
              << obj.calldata_hash << "\n";
}

} // namespace aztec3::circuits::abis