#pragma once

#include "private_call_data.hpp"
#include "../tx_request.hpp"

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/native_types.hpp"

namespace aztec3::circuits::abis::private_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct PrivateKernelInputsInit {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    TxRequest<NCT> tx_request{};
    PrivateCallData<NCT> private_call{};

    // For serialization, update with new fields
    MSGPACK_FIELDS(tx_request, private_call);
    boolean operator==(PrivateKernelInputsInit<NCT> const& other) const
    {
        return tx_request == other.tx_request && private_call == other.private_call;
    };

    template <typename Builder> PrivateKernelInputsInit<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        PrivateKernelInputsInit<CircuitTypes<Builder>> private_inputs = {
            // TODO to_ct(signature),
            tx_request.to_circuit_type(builder),
            private_call.to_circuit_type(builder),
        };

        return private_inputs;
    };
};

}  // namespace aztec3::circuits::abis::private_kernel
