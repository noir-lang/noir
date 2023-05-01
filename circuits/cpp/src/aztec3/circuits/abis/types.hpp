#pragma once
#include "private_circuit_public_inputs.hpp"
#include "public_circuit_public_inputs.hpp"

#include <aztec3/constants.hpp>
#include <aztec3/utils/array.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/native_types.hpp>

#include <barretenberg/common/map.hpp>
#include <barretenberg/crypto/generators/generator_data.hpp>
#include <barretenberg/stdlib/hash/pedersen/pedersen.hpp>
#include <barretenberg/stdlib/primitives/witness/witness.hpp>

namespace aztec3::circuits::abis {

template <typename NCT> struct PrivateTypes {
    using AppCircuitPublicInputs = PrivateCircuitPublicInputs<NCT>;
};

template <typename NCT> struct PublicTypes {
    using AppCircuitPublicInputs = PublicCircuitPublicInputs<NCT>;
};
}  // namespace aztec3::circuits::abis