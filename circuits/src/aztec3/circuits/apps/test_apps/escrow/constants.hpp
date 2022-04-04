// #pragma once
// #include "init.hpp"
// #include <aztec3/circuits/apps/contract_factory.hpp>
// #include <aztec3/circuits/apps/function.hpp>

// namespace aztec3::circuits::apps::test_apps::escrow {

// using plonk::stdlib::witness_t;
// CT::boolean TRUE;
// CT::boolean FALSE;
// CT::fr ZERO;

// void init(Composer& composer)
// {
//     TRUE = witness_t(&composer, true);
//     TRUE.assert_equal(CT::boolean(true));

//     FALSE = witness_t(&composer, false);
//     FALSE.assert_equal(CT::boolean(false));

//     ZERO = witness_t(&composer, 0);
//     ZERO.assert_equal(CT::fr(0));
// }

// } // namespace aztec3::circuits::apps::test_apps::escrow