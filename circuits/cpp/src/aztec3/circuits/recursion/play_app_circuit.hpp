#pragma once
#include "init.hpp"

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/native_types.hpp"

namespace aztec3::circuits::recursion {

template <typename Composer> void play_app_circuit(Composer& composer, NT::fr const& a_in, NT::fr const& b_in)
{
    CT::fr const a = CT::witness(&composer, a_in);
    CT::fr const b = CT::witness(&composer, b_in);
    CT::fr const c = a * b;
    CT::fr const d = a + b + c;

    a.set_public();
    d.set_public();
};

}  // namespace aztec3::circuits::recursion