/**
 * @brief Contains all the headers required to adequately compile the types defined in circuit_builders_fwd.hpp and
 * instantiate templates.
 */
#pragma once
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"

template <typename T>
concept HasPlookup =
    proof_system::IsAnyOf<T, proof_system::UltraCircuitBuilder, proof_system::GoblinUltraCircuitBuilder>;

template <typename T>
concept IsGoblinBuilder = proof_system::IsAnyOf<T, proof_system::GoblinUltraCircuitBuilder>;
template <typename T>
concept IsNotGoblinBuilder = !
IsGoblinBuilder<T>;
