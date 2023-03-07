#include <stdlib/types/types.hpp>

namespace aztec3::circuits::recursion {

using namespace plonk::stdlib::types;
// using plonk::stdlib::recursion::recursion_output;

void play_app_circuit(Composer& composer, barretenberg::fr const& a_in, barretenberg::fr const& b_in)
{

    field_ct a = witness_ct(&composer, a_in);
    field_ct b = witness_ct(&composer, b_in);
    field_ct c = a * b;
    field_ct d = a + b + c;

    a.set_public();
    d.set_public();
};

} // namespace aztec3::circuits::recursion