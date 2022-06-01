#pragma once
#include <common/map.hpp>
#include <stdlib/primitives/field/field.hpp>
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <aztec3/circuits/abis/private_kernel/public_inputs.hpp>
// #include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>

namespace aztec3::circuits::mock {

// *****************************************

// MIKE! TODO! This is the wrong PublicInputs class to use! Use the private kernel's PublicINputs!!!

//*******************************************

using aztec3::circuits::abis::private_kernel::PublicInputs;
using NT = plonk::stdlib::types::NativeTypes;
using plonk::stdlib::pedersen;
using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;

template <typename Composer> void mock_circuit_2(Composer& composer, PublicInputs<NT> const& _public_inputs)
{
    typedef CircuitTypes<Composer> CT;
    auto public_inputs = _public_inputs.to_circuit_type(composer);
    public_inputs.set_public();

    plonk::stdlib::pedersen<Composer>::compress(typename CT::fr(witness_t(&composer, 1)),
                                                typename CT::fr(witness_t(&composer, 1)));
}

} // namespace aztec3::circuits::mock
