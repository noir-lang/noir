#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include "barretenberg/stdlib/primitives/bool/bool.hpp"

template <typename Composer> class Add2Circuit {
  public:
    typedef stdlib::field_t<Composer> field_ct;
    typedef stdlib::witness_t<Composer> witness_ct;
    typedef stdlib::public_witness_t<Composer> public_witness_ct;

    // Three public inputs
    static Composer generate(std::string srs_path, uint256_t inputs[])
    {

        Composer composer(srs_path);

        field_ct a(public_witness_ct(&composer, inputs[0]));
        field_ct b(public_witness_ct(&composer, inputs[1]));
        field_ct c(public_witness_ct(&composer, inputs[2]));
        c.assert_equal(a + b);

        return composer;
    }
};