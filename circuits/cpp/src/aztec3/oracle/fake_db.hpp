#pragma once

#include <aztec3/circuits/abis/call_context.hpp>

#include <aztec3/circuits/apps/utxo_datum.hpp>

#include <aztec3/circuits/apps/notes/default_private_note/note_preimage.hpp>
#include <aztec3/circuits/apps/notes/default_singleton_private_note/note_preimage.hpp>

#include <stdlib/types/native_types.hpp>

namespace aztec3::oracle {

using aztec3::circuits::abis::CallContext;

using aztec3::circuits::apps::UTXOSLoadDatum;

using aztec3::circuits::apps::notes::DefaultPrivateNotePreimage;

using aztec3::circuits::apps::notes::DefaultSingletonPrivateNotePreimage;

using plonk::stdlib::types::CircuitTypes;
using NT = plonk::stdlib::types::NativeTypes;

// A temporary stub, whilst building other things first.
class FakeDB {
  public:
    FakeDB(){};

    /**
     * For getting a singleton UTXO (not a set).
     *
     * NOTICE: this fake db stub is hard-coded to a DefaultPrivateNotePreimage which _itself_ is hard-coded to the value
     * type being a field.
     * So if you want to test other note types against this stub DB, you'll need to write your own stub DB entry.
     */
    UTXOSLoadDatum<NT, DefaultPrivateNotePreimage<NT, typename NT::fr>> get_utxo_sload_datum(
        NT::address const& contract_address,
        NT::grumpkin_point const& storage_slot_point,
        DefaultPrivateNotePreimage<NT, typename NT::fr> const& advice)
    // NT::address const& owner,
    // NT::fr required_utxo_tree_root,
    // size_t utxo_tree_depth)
    {
        (void)storage_slot_point; // Not used in this 'fake' implementation.

        DefaultPrivateNotePreimage<NT, NT::fr> preimage{
            .value = 100,
            .owner = advice.owner,
            .creator_address = 0,
            .memo = 3456,
            .salt = 1234,
            .nonce = 2345,
            .is_dummy = false,
        };

        const size_t utxo_tree_depth = 32;
        const NT::fr required_utxo_tree_root = 2468;

        std::vector<NT::fr> sibling_path(utxo_tree_depth);
        std::fill(sibling_path.begin(), sibling_path.end(), 1); // Fill with 1's to be lazy. TODO: return a valid path.

        return {
            .commitment = 1,
            .contract_address = contract_address,
            .preimage = preimage,

            .sibling_path = sibling_path,
            .leaf_index = 2,
            .old_private_data_tree_root = required_utxo_tree_root,
        };
    };

    /**
     * For getting a set of UTXOs.
     *
     * * NOTICE: this fake db stub is hard-coded to a DefaultPrivateNotePreimage which _itself_ is hard-coded to the
     * value type being a field.
     * So if you want to test other note types against this stub DB, you'll need to write your own stub DB entry.
     */
    std::vector<UTXOSLoadDatum<NT, DefaultPrivateNotePreimage<NT, typename NT::fr>>> get_utxo_sload_data(
        NT::address const& contract_address,
        NT::grumpkin_point const& storage_slot_point,
        size_t const& num_notes,
        DefaultPrivateNotePreimage<NT, typename NT::fr> const& advice)
    // NT::address const& owner,
    // NT::fr required_utxo_tree_root,
    // size_t utxo_tree_depth)
    {
        (void)storage_slot_point; // Not used in this 'fake' implementation.

        std::vector<UTXOSLoadDatum<NT, DefaultPrivateNotePreimage<NT, typename NT::fr>>> data;

        const size_t utxo_tree_depth = 32;
        const NT::fr required_utxo_tree_root = 2468;

        std::vector<NT::fr> sibling_path(utxo_tree_depth);
        std::fill(sibling_path.begin(), sibling_path.end(), 1); // Fill with 1's to be lazy. TODO: return a valid path.

        for (size_t i = 0; i < num_notes; i++) {
            DefaultPrivateNotePreimage<NT, NT::fr> preimage{
                .value = 100 + i,
                .owner = advice.owner,
                .creator_address = 0,
                .memo = 3456,
                .salt = 1234,
                .nonce = 2345,
                .is_dummy = false,
            };

            data.push_back({
                .commitment = 1,
                .contract_address = contract_address,
                .preimage = preimage,

                .sibling_path = sibling_path,
                .leaf_index = 2,
                .old_private_data_tree_root = required_utxo_tree_root,
            });
        }

        return data;
    };

    /**
     * For getting a singleton UTXO (not a set).
     *
     * NOTICE: this fake db stub is hard-coded to a DefaultSingletonPrivateNotePreimage which _itself_ is hard-coded to
     * the value type being a field. So if you want to test other note types against this stub DB, you'll need to write
     * your own stub DB entry.
     */
    UTXOSLoadDatum<NT, DefaultSingletonPrivateNotePreimage<NT, typename NT::fr>> get_utxo_sload_datum(
        NT::address const& contract_address,
        NT::grumpkin_point const& storage_slot_point,
        DefaultSingletonPrivateNotePreimage<NT, typename NT::fr> const& advice)
    // NT::address const& owner,
    // NT::fr required_utxo_tree_root,
    // size_t utxo_tree_depth)
    {
        (void)storage_slot_point; // Not used in this 'fake' implementation.

        DefaultSingletonPrivateNotePreimage<NT, NT::fr> preimage{
            .value = 100,
            .owner = advice.owner,
            .salt = 1234,
            .nonce = 2345,
        };

        const size_t utxo_tree_depth = 32;
        const NT::fr required_utxo_tree_root = 2468;

        std::vector<NT::fr> sibling_path(utxo_tree_depth);
        std::fill(sibling_path.begin(), sibling_path.end(), 1); // Fill with 1's to be lazy. TODO: return a valid path.

        return {
            .commitment = 1,
            .contract_address = contract_address,
            .preimage = preimage,

            .sibling_path = sibling_path,
            .leaf_index = 2,
            .old_private_data_tree_root = required_utxo_tree_root,
        };
    };
};

} // namespace aztec3::oracle