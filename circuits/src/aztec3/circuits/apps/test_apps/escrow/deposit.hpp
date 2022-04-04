#pragma once
#include "init.hpp"

namespace aztec3::circuits::apps::test_apps::escrow {

void deposit(
    Composer& composer, OracleWrapper& oracle, NT::fr const& _amount, NT::fr const& _asset_id, NT::fr const& _memo);

} // namespace aztec3::circuits::apps::test_apps::escrow