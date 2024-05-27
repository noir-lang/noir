#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "circuit_builder_base_impl.hpp"

namespace bb {

// Standard honk/ plonk instantiation
template class CircuitBuilderBase<bb::fr>;
template class CircuitBuilderBase<grumpkin::fr>;
} // namespace bb
