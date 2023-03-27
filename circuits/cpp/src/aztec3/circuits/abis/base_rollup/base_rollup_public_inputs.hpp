#pragma once
#include <barretenberg/stdlib/recursion/aggregation_state/aggregation_state.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include "../append_only_tree_snapshot.hpp"
#include "../append_only_tree_snapshot.hpp"
#include "./constant_base_rollup_data.hpp"

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

const uint32_t BASE_ROLLUP_TYPE = 0;
const uint32_t MERGE_ROLLUP_TYPE = 1;

template <typename NCT> struct BaseRollupPublicInputs {
    typedef typename NCT::fr fr;
    typedef typename NCT::AggregationObject AggregationObject;

    uint32_t rollup_type;

    AggregationObject end_aggregation_object;

    ConstantBaseRollupData<NCT> constants;

    // The only tree root actually updated in this circuit is the nullifier tree, because earlier leaves (of
    // low_nullifiers) must be updated to point to the new nullifiers in this circuit.
    AppendOnlyTreeSnapshot<NCT> start_nullifier_tree_snapshot;
    AppendOnlyTreeSnapshot<NCT> end_nullifier_tree_snapshots;

    fr new_commitments_subtree_root;
    fr new_nullifiers_subtree_root;
    fr new_contract_leaves_subtree_root;

    // Hashes (probably sha256) to make public inputs constant-sized (to then be unpacked on-chain)
    // UPDATE we should instead just hash all of the below into a single value. See big diagram of sha256 hashing
    // bottom-right of here.
    // TODO I've put `fr`, but these hash values' types might need to be two fields if we want all 256-bits, for
    // security purposes.
    fr new_commitments_hash;
    fr new_nullifiers_hash;
    fr new_l1_msgs_hash;
    fr new_contract_data_hash;
    fr prover_contributions_hash;

    bool operator==(BaseRollupPublicInputs<NCT> const&) const = default;
};

template <typename NCT> void read(uint8_t const*& it, BaseRollupPublicInputs<NCT>& obj)
{
    using serialize::read;

    read(it, obj.rollup_type);
    read(it, obj.end_aggregation_object);
    read(it, obj.constants);
    read(it, obj.start_nullifier_tree_snapshot);
    read(it, obj.end_nullifier_tree_snapshots);
    read(it, obj.new_commitments_subtree_root);
    read(it, obj.new_nullifiers_subtree_root);
    read(it, obj.new_contract_leaves_subtree_root);
    read(it, obj.new_commitments_hash);
    read(it, obj.new_nullifiers_hash);
    read(it, obj.new_l1_msgs_hash);
    read(it, obj.new_contract_data_hash);
    read(it, obj.prover_contributions_hash);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, BaseRollupPublicInputs<NCT> const& obj)
{
    using serialize::write;

    write(buf, obj.rollup_type);
    write(buf, obj.end_aggregation_object);
    write(buf, obj.constants);
    write(buf, obj.start_nullifier_tree_snapshot);
    write(buf, obj.end_nullifier_tree_snapshots);
    write(buf, obj.new_commitments_subtree_root);
    write(buf, obj.new_nullifiers_subtree_root);
    write(buf, obj.new_contract_leaves_subtree_root);
    write(buf, obj.new_commitments_hash);
    write(buf, obj.new_nullifiers_hash);
    write(buf, obj.new_l1_msgs_hash);
    write(buf, obj.new_contract_data_hash);
    write(buf, obj.prover_contributions_hash);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, BaseRollupPublicInputs<NCT> const& obj)
{
    return os << "rollup_type: " << obj.rollup_type
              << "\n"
                 "end_aggregation_object:\n"
              << obj.end_aggregation_object
              << "\n"
                 "constants:\n"
              << obj.constants
              << "\n"
                 "start_nullifier_tree_snapshot:\n"
              << obj.start_nullifier_tree_snapshot
              << "\n"
                 "end_nullifier_tree_snapshots:\n"
              << obj.end_nullifier_tree_snapshots
              << "\n"
                 "new_commitments_subtree_root: "
              << obj.new_commitments_subtree_root
              << "\n"
                 "new_nullifiers_subtree_root: "
              << obj.new_nullifiers_subtree_root
              << "\n"
                 "new_contract_leaves_subtree_root: "
              << obj.new_contract_leaves_subtree_root
              << "\n"
                 "new_commitments_hash: "
              << obj.new_commitments_hash
              << "\n"
                 "new_nullifiers_hash: "
              << obj.new_nullifiers_hash
              << "\n"
                 "new_l1_msgs_hash: "
              << obj.new_l1_msgs_hash
              << "\n"
                 "new_contract_data_hash: "
              << obj.new_contract_data_hash
              << "\n"
                 "prover_contributions_hash: "
              << obj.prover_contributions_hash << "\n";
}

} // namespace aztec3::circuits::abis