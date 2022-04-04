#pragma once
#include "init.hpp"
#include <aztec3/circuits/apps/private_state_factory.hpp>
#include <aztec3/circuits/apps/private_state_var.hpp>

namespace aztec3::circuits::apps::test_apps::escrow {

inline PrivateStateFactory<Composer> init_states(Composer& composer, OracleWrapper& oracle)
{
    PrivateStateFactory<Composer> private_state_factory(composer, oracle, "Escrow");

    private_state_factory.new_private_state("balances", { "owner", "asset_id" });

    return private_state_factory;
}

} // namespace aztec3::circuits::apps::test_apps::escrow