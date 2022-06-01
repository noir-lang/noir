#pragma once
#include "init.hpp"
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::PrivateCircuitPublicInputs;

PrivateCircuitPublicInputs<NT> deposit(
    Composer& composer, OracleWrapper& oracle, NT::fr const& _amount, NT::fr const& _asset_id, NT::fr const& _memo);

} // namespace aztec3::circuits::apps::test_apps::escrow