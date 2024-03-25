/**
 * @brief Contains all the headers required to adequately compile the types defined in circuit_builders_fwd.hpp and
 * instantiate templates.
 */
#pragma once
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/stdlib_circuit_builders/standard_circuit_builder.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"

template <typename T>
concept HasPlookup = bb::IsAnyOf<T, bb::UltraCircuitBuilder, bb::GoblinUltraCircuitBuilder>;

template <typename T>
concept IsStandardBuilder = bb::IsAnyOf<T, bb::StandardCircuitBuilder_<bb::fr>, bb::StandardCircuitBuilder_<bb::fq>>;
template <typename T>
concept IsUltraBuilder = bb::IsAnyOf<T, bb::UltraCircuitBuilder, bb::GoblinUltraCircuitBuilder>;

template <typename T>
concept IsGoblinBuilder = bb::IsAnyOf<T, bb::GoblinUltraCircuitBuilder>;
template <typename T>
concept IsNotGoblinBuilder = !
IsGoblinBuilder<T>;
