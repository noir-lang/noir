
#pragma once

#include "barretenberg/crypto/sha256/sha256.hpp"

#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <ostream>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct RootRollupPublicInputs {
    typedef typename NCT::fr fr;
    typedef typename NCT::AggregationObject AggregationObject;

    // All below are shared between the base and merge rollups
    AggregationObject end_aggregation_object;

    AppendOnlyTreeSnapshot<NCT> start_private_data_tree_snapshot;
    AppendOnlyTreeSnapshot<NCT> end_private_data_tree_snapshot;

    AppendOnlyTreeSnapshot<NCT> start_nullifier_tree_snapshot;
    AppendOnlyTreeSnapshot<NCT> end_nullifier_tree_snapshot;

    AppendOnlyTreeSnapshot<NCT> start_contract_tree_snapshot;
    AppendOnlyTreeSnapshot<NCT> end_contract_tree_snapshot;

    AppendOnlyTreeSnapshot<NCT> start_tree_of_historic_private_data_tree_roots_snapshot;
    AppendOnlyTreeSnapshot<NCT> end_tree_of_historic_private_data_tree_roots_snapshot;

    AppendOnlyTreeSnapshot<NCT> start_tree_of_historic_contract_tree_roots_snapshot;
    AppendOnlyTreeSnapshot<NCT> end_tree_of_historic_contract_tree_roots_snapshot;

    std::array<fr, 2> calldata_hash;

    bool operator==(RootRollupPublicInputs<NCT> const&) const = default;

    fr hash() const
    {
        std::vector<uint8_t> buf;

        write(buf, start_private_data_tree_snapshot);
        write(buf, start_nullifier_tree_snapshot);
        write(buf, start_contract_tree_snapshot);
        write(buf, start_tree_of_historic_private_data_tree_roots_snapshot);
        write(buf, start_tree_of_historic_contract_tree_roots_snapshot);
        write(buf, end_private_data_tree_snapshot);
        write(buf, end_nullifier_tree_snapshot);
        write(buf, end_contract_tree_snapshot);
        write(buf, end_tree_of_historic_private_data_tree_roots_snapshot);
        write(buf, end_tree_of_historic_contract_tree_roots_snapshot);

        // Stitching calldata hash together
        auto high_buffer = calldata_hash[0].to_buffer();
        auto low_buffer = calldata_hash[1].to_buffer();

        for (uint8_t i = 0; i < 16; i++) {
            buf.push_back(high_buffer[16 + i]);
        }
        for (uint8_t i = 0; i < 16; i++) {
            buf.push_back(low_buffer[16 + i]);
        }

        return sha256::sha256_to_field(buf);
    }
};

template <typename NCT> void read(uint8_t const*& it, RootRollupPublicInputs<NCT>& obj)
{
    using serialize::read;

    read(it, obj.end_aggregation_object);
    read(it, obj.start_private_data_tree_snapshot);
    read(it, obj.end_private_data_tree_snapshot);
    read(it, obj.start_nullifier_tree_snapshot);
    read(it, obj.end_nullifier_tree_snapshot);
    read(it, obj.start_contract_tree_snapshot);
    read(it, obj.end_contract_tree_snapshot);
    read(it, obj.start_tree_of_historic_private_data_tree_roots_snapshot);
    read(it, obj.end_tree_of_historic_private_data_tree_roots_snapshot);
    read(it, obj.start_tree_of_historic_contract_tree_roots_snapshot);
    read(it, obj.end_tree_of_historic_contract_tree_roots_snapshot);
    read(it, obj.calldata_hash);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, RootRollupPublicInputs<NCT> const& obj)
{
    using serialize::write;

    write(buf, obj.end_aggregation_object);
    write(buf, obj.start_private_data_tree_snapshot);
    write(buf, obj.end_private_data_tree_snapshot);
    write(buf, obj.start_nullifier_tree_snapshot);
    write(buf, obj.end_nullifier_tree_snapshot);
    write(buf, obj.start_contract_tree_snapshot);
    write(buf, obj.end_contract_tree_snapshot);
    write(buf, obj.start_tree_of_historic_private_data_tree_roots_snapshot);
    write(buf, obj.end_tree_of_historic_private_data_tree_roots_snapshot);
    write(buf, obj.start_tree_of_historic_contract_tree_roots_snapshot);
    write(buf, obj.end_tree_of_historic_contract_tree_roots_snapshot);
    write(buf, obj.calldata_hash);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, RootRollupPublicInputs<NCT> const& obj)
{
    return os << "end_aggregation_object: " << obj.end_aggregation_object << "\n"
              << "start_private_data_tree_snapshot: " << obj.start_private_data_tree_snapshot << "\n"
              << "end_private_data_tree_snapshot: " << obj.end_private_data_tree_snapshot << "\n"
              << "start_nullifier_tree_snapshot: " << obj.start_nullifier_tree_snapshot << "\n"
              << "end_nullifier_tree_snapshot: " << obj.end_nullifier_tree_snapshot << "\n"
              << "start_contract_tree_snapshot: " << obj.start_contract_tree_snapshot << "\n"
              << "end_contract_tree_snapshot: " << obj.end_contract_tree_snapshot << "\n"
              << "start_tree_of_historic_private_data_tree_roots_snapshot: "
              << obj.start_tree_of_historic_private_data_tree_roots_snapshot << "\n"
              << "end_tree_of_historic_private_data_tree_roots_snapshot: "
              << obj.end_tree_of_historic_private_data_tree_roots_snapshot << "\n"
              << "start_tree_of_historic_contract_tree_roots_snapshot: "
              << obj.start_tree_of_historic_contract_tree_roots_snapshot << "\n"
              << "end_tree_of_historic_contract_tree_roots_snapshot: "
              << obj.end_tree_of_historic_contract_tree_roots_snapshot << "\n"
              << "calldata_hash: " << obj.calldata_hash << "\n";
};

} // namespace aztec3::circuits::abis