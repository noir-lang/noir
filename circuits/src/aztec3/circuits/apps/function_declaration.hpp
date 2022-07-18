#pragma once
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::apps {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct FunctionDeclaration {
    typedef typename NCT::boolean boolean;

    std::string name;
    boolean is_private = false;
    boolean is_constructor = false;
};

} // namespace aztec3::circuits::apps