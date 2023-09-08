#pragma once

#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include "aztec3/circuits/abis/global_variables.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

#include <ostream>

namespace aztec3::circuits::abis {

template <typename NCT> struct RootRollupPublicInputs {
    using fr = typename NCT::fr;
    using AggregationObject = typename NCT::AggregationObject;

    // All below are shared between the base and merge rollups
    AggregationObject end_aggregation_object{};

    GlobalVariables<NCT> global_variables{};

    AppendOnlyTreeSnapshot<NCT> start_private_data_tree_snapshot{};
    AppendOnlyTreeSnapshot<NCT> end_private_data_tree_snapshot{};

    AppendOnlyTreeSnapshot<NCT> start_nullifier_tree_snapshot{};
    AppendOnlyTreeSnapshot<NCT> end_nullifier_tree_snapshot{};

    AppendOnlyTreeSnapshot<NCT> start_contract_tree_snapshot{};
    AppendOnlyTreeSnapshot<NCT> end_contract_tree_snapshot{};

    fr start_public_data_tree_root{};
    fr end_public_data_tree_root{};

    AppendOnlyTreeSnapshot<NCT> start_tree_of_historic_private_data_tree_roots_snapshot{};
    AppendOnlyTreeSnapshot<NCT> end_tree_of_historic_private_data_tree_roots_snapshot{};

    AppendOnlyTreeSnapshot<NCT> start_tree_of_historic_contract_tree_roots_snapshot{};
    AppendOnlyTreeSnapshot<NCT> end_tree_of_historic_contract_tree_roots_snapshot{};

    AppendOnlyTreeSnapshot<NCT> start_l1_to_l2_messages_tree_snapshot{};
    AppendOnlyTreeSnapshot<NCT> end_l1_to_l2_messages_tree_snapshot{};

    AppendOnlyTreeSnapshot<NCT> start_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot{};
    AppendOnlyTreeSnapshot<NCT> end_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot{};

    AppendOnlyTreeSnapshot<NCT> start_historic_blocks_tree_snapshot{};
    AppendOnlyTreeSnapshot<NCT> end_historic_blocks_tree_snapshot{};

    std::array<fr, NUM_FIELDS_PER_SHA256> calldata_hash{};
    std::array<fr, NUM_FIELDS_PER_SHA256> l1_to_l2_messages_hash{};

    // For serialization, update with new fields
    MSGPACK_FIELDS(end_aggregation_object,
                   global_variables,
                   start_private_data_tree_snapshot,
                   end_private_data_tree_snapshot,
                   start_nullifier_tree_snapshot,
                   end_nullifier_tree_snapshot,
                   start_contract_tree_snapshot,
                   end_contract_tree_snapshot,
                   start_public_data_tree_root,
                   end_public_data_tree_root,
                   start_tree_of_historic_private_data_tree_roots_snapshot,
                   end_tree_of_historic_private_data_tree_roots_snapshot,
                   start_tree_of_historic_contract_tree_roots_snapshot,
                   end_tree_of_historic_contract_tree_roots_snapshot,
                   start_l1_to_l2_messages_tree_snapshot,
                   end_l1_to_l2_messages_tree_snapshot,
                   start_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot,
                   end_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot,
                   start_historic_blocks_tree_snapshot,
                   end_historic_blocks_tree_snapshot,
                   calldata_hash,
                   l1_to_l2_messages_hash);

    bool operator==(RootRollupPublicInputs<NCT> const&) const = default;

    fr hash() const
    {
        std::vector<uint8_t> buf;

        write(&buf, global_variables);
        write(buf, start_private_data_tree_snapshot);
        write(buf, start_nullifier_tree_snapshot);
        write(buf, start_contract_tree_snapshot);
        write(buf, start_tree_of_historic_private_data_tree_roots_snapshot);
        write(buf, start_tree_of_historic_contract_tree_roots_snapshot);
        write(buf, start_public_data_tree_root);
        write(buf, start_l1_to_l2_messages_tree_snapshot);
        write(buf, start_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot);
        write(buf, start_historic_blocks_tree_snapshot);
        write(buf, end_private_data_tree_snapshot);
        write(buf, end_nullifier_tree_snapshot);
        write(buf, end_contract_tree_snapshot);
        write(buf, end_tree_of_historic_private_data_tree_roots_snapshot);
        write(buf, end_tree_of_historic_contract_tree_roots_snapshot);
        write(buf, end_public_data_tree_root);
        write(buf, end_l1_to_l2_messages_tree_snapshot);
        write(buf, end_tree_of_historic_l1_to_l2_messages_tree_roots_snapshot);
        write(buf, end_historic_blocks_tree_snapshot);

        // Stitching calldata hash together
        auto high_buffer = calldata_hash[0].to_buffer();
        auto low_buffer = calldata_hash[1].to_buffer();

        for (uint8_t i = 0; i < 16; i++) {
            buf.push_back(high_buffer[16 + i]);
        }
        for (uint8_t i = 0; i < 16; i++) {
            buf.push_back(low_buffer[16 + i]);
        }

        // Stitch l1_to_l2_messages_hash
        auto high_buffer_m = l1_to_l2_messages_hash[0].to_buffer();
        auto low_buffer_m = l1_to_l2_messages_hash[1].to_buffer();

        for (uint8_t i = 0; i < 16; i++) {
            buf.push_back(high_buffer_m[16 + i]);
        }
        for (uint8_t i = 0; i < 16; i++) {
            buf.push_back(low_buffer_m[16 + i]);
        }

        return sha256::sha256_to_field(buf);
    }
};

}  // namespace aztec3::circuits::abis
