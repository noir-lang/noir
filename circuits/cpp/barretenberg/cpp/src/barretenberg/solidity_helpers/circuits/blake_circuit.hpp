#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
#include "barretenberg/stdlib/hash/blake2s/blake2s.hpp"

using namespace proof_system::plonk;
using namespace proof_system::plonk::stdlib;

using numeric::uint256_t;

template <typename Composer> class BlakeCircuit {
  public:
    typedef stdlib::field_t<Composer> field_ct;
    typedef stdlib::public_witness_t<Composer> public_witness_ct;
    typedef stdlib::byte_array<Composer> byte_array_ct;

    static constexpr size_t NUM_PUBLIC_INPUTS = 4;

    static Composer generate(std::string srs_path, uint256_t public_inputs[])
    {
        Composer composer(srs_path);

        byte_array_ct input_buffer(&composer);
        for (size_t i = 0; i < NUM_PUBLIC_INPUTS; ++i) {
            input_buffer.write(byte_array_ct(field_ct(public_witness_ct(&composer, public_inputs[i]))));
        }

        stdlib::blake2s<Composer>(input_buffer);

        return composer;
    }
};