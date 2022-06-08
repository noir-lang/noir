#pragma once
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
#include "accumulated_data.hpp"
#include "previous_kernel_data.hpp"
#include "private_call_data.hpp"

namespace aztec3::circuits::abis::private_kernel {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;
using std::is_same;

template <typename NCT> struct PrivateInputs {
    typedef typename NCT::fr fr;

    // Signature signature; // TODO!
    // AccumulatedData<NCT> start;
    PreviousKernelData<NCT> previous_kernel;
    PrivateCallData<NCT> private_call;

    template <typename Composer> PrivateInputs<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        PrivateInputs<CircuitTypes<Composer>> private_inputs = {
            // signature.to_circuit_type();
            // start.to_circuit_type(composer),
            previous_kernel.to_circuit_type(composer),
            private_call.to_circuit_type(composer),
        };

        return private_inputs;
    };
};

} // namespace aztec3::circuits::abis::private_kernel