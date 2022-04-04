#pragma once
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::apps {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

// struct L1ResultArgIndex {
//     size_t arg_index;
// };

/**
 * This just allows some syntactic sugar when writing test circuits.
 * This is really just something which returns the index you feed it via `[]`.
 */
class L1Result {
  public:
    L1Result() {}

    size_t operator[](size_t i) const { return i; }
};

} // namespace aztec3::circuits::apps