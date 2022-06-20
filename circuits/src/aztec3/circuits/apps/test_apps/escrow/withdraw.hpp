#pragma once
#include "init.hpp"
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> withdraw(Composer& composer,
                                                OracleWrapper& oracle,
                                                NT::fr const& _amount,
                                                NT::fr const& _asset_id,
                                                NT::fr const& _memo,
                                                NT::fr const& _l1_withdrawal_address,
                                                NT::fr const& _fee);

} // namespace aztec3::circuits::apps::test_apps::escrow