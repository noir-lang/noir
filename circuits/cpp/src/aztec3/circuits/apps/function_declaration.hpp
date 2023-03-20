#pragma once
#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::apps {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

// This exists just so that designated initialisers can be used when passing this info to a function, for readability.
template <typename NCT> struct FunctionDeclaration {
    typedef typename NCT::boolean boolean;

    std::string name;
    boolean is_private = false;
    boolean is_constructor = false;
};

} // namespace aztec3::circuits::apps