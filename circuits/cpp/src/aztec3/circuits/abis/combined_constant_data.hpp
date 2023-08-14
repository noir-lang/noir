#pragma once

#include "tx_context.hpp"

#include "aztec3/circuits/abis/historic_block_data.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::circuits::abis::HistoricBlockData;
using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct CombinedConstantData {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    HistoricBlockData<NCT> block_data{};
    TxContext<NCT> tx_context{};

    // for serialization: update up with new fields
    MSGPACK_FIELDS(block_data, tx_context);
    boolean operator==(CombinedConstantData<NCT> const& other) const
    {
        return block_data == other.block_data && tx_context == other.tx_context;
    }

    template <typename Builder> CombinedConstantData<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        CombinedConstantData<CircuitTypes<Builder>> constant_data = {
            block_data.to_circuit_type(builder),
            tx_context.to_circuit_type(builder),
        };

        return constant_data;
    };

    template <typename Builder> CombinedConstantData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);

        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Builder>(); };

        CombinedConstantData<NativeTypes> constant_data = {
            to_native_type(block_data),
            to_native_type(tx_context),
        };

        return constant_data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        block_data.set_public();
        tx_context.set_public();
    }
};

}  // namespace aztec3::circuits::abis
