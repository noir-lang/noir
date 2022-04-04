#pragma once
#include "init.hpp"

namespace aztec3::circuits::apps::test_apps::escrow {

void transfer(Composer& composer,
              OracleWrapper& oracle,
              NT::fr const& _amount,
              NT::address const& _to,
              NT::fr const& _asset_id,
              NT::fr const& _memo,
              NT::boolean const& _reveal_msg_sender_to_recipient,
              NT::fr const& _fee,
              NT::boolean const& _is_fee_payment);

} // namespace aztec3::circuits::apps::test_apps::escrow