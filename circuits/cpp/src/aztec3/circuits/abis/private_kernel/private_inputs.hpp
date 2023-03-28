#pragma once

#include "accumulated_data.hpp"
#include "previous_kernel_data.hpp"
#include "private_call_data.hpp"
#include "../signed_tx_request.hpp"

#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::abis::private_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;
using std::is_same;

template <typename NCT> struct PrivateInputs {
    typedef typename NCT::fr fr;
    // typedef typename NCT::signature Signature;

    SignedTxRequest<NCT> signed_tx_request;
    PreviousKernelData<NCT> previous_kernel;
    PrivateCallData<NCT> private_call;

    template <typename Composer> PrivateInputs<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        PrivateInputs<CircuitTypes<Composer>> private_inputs = {
            // TODO to_ct(signature),
            signed_tx_request.to_circuit_type(composer),
            previous_kernel.to_circuit_type(composer),
            private_call.to_circuit_type(composer),
        };

        return private_inputs;
    };
};

} // namespace aztec3::circuits::abis::private_kernel