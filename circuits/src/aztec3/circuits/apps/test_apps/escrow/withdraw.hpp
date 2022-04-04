#pragma once
#include "init.hpp"

namespace aztec3::circuits::apps::test_apps::escrow {

void withdraw(Composer& composer,
              OracleWrapper& oracle,
              NT::fr const& _amount,
              NT::fr const& _asset_id,
              NT::fr const& _memo,
              NT::fr const& _l1_withdrawal_address,
              NT::fr const& _fee,
              NT::boolean const& _is_fee_payment);

} // namespace aztec3::circuits::apps::test_apps::escrow