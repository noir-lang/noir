#pragma once

#include "accumulated_data.hpp"
#include "previous_kernel_data.hpp"
#include "private_call_data.hpp"
#include "../signed_tx_object.hpp"

#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::abis::private_kernel {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;
using std::is_same;

template <typename NCT> struct PrivateInputs {
    typedef typename NCT::fr fr;
    // typedef typename NCT::signature Signature;

    SignedTxObject<NCT> signed_tx_object;
    PreviousKernelData<NCT> previous_kernel;
    PrivateCallData<NCT> private_call;

    template <typename Composer> PrivateInputs<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        PrivateInputs<CircuitTypes<Composer>> private_inputs = {
            // plonk::stdlib::schnorr::convert_signature(&composer, signature),
            signed_tx_object.to_circuit_type(composer),
            previous_kernel.to_circuit_type(composer),
            private_call.to_circuit_type(composer),
        };

        return private_inputs;
    };
};

} // namespace aztec3::circuits::abis::private_kernel