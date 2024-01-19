/**
 * @brief Contains all the headers required to adequately compile the types defined in circuit_builders_fwd.hpp and
 * instantiate templates.
 */
#pragma once
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"

template <typename T>
concept HasPlookup = bb::IsAnyOf<T, bb::UltraCircuitBuilder, bb::GoblinUltraCircuitBuilder>;

template <typename T>
concept IsGoblinBuilder = bb::IsAnyOf<T, bb::GoblinUltraCircuitBuilder>;
template <typename T>
concept IsNotGoblinBuilder = !
IsGoblinBuilder<T>;
