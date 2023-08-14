#pragma once
#include "function_data.hpp"
#include "tx_context.hpp"

#include "aztec3/utils/array.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct TxRequest {
    using address = typename NCT::address;
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    address origin = 0;
    FunctionData<NCT> function_data{};
    fr args_hash = 0;
    TxContext<NCT> tx_context{};

    // For serialization, update with new fields
    MSGPACK_FIELDS(origin, function_data, args_hash, tx_context);
    boolean operator==(TxContext<NCT> const& other) const
    {
        return origin == other.origin && function_data == other.function_data && args_hash == other.args &&
               tx_context == other.tx_context;
    };

    template <typename Builder> TxRequest<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(builder); };

        TxRequest<CircuitTypes<Builder>> tx_request = {
            to_ct(origin),
            to_circuit_type(function_data),
            to_ct(args_hash),
            to_circuit_type(tx_context),
        };

        return tx_request;
    };

    fr hash() const
    {
        std::vector<fr> inputs;
        inputs.push_back(fr(origin));
        inputs.push_back(function_data.hash());
        inputs.push_back(args_hash);
        inputs.push_back(tx_context.hash());

        return NCT::compress(inputs, GeneratorIndex::TX_REQUEST);
    }
};

}  // namespace aztec3::circuits::abis
