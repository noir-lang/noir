#pragma once
#include "combined_accumulated_data.hpp"
#include "combined_constant_data.hpp"
#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;
using std::conditional;
using std::is_same;

template <typename NCT> struct KernelCircuitPublicInputs {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    CombinedAccumulatedData<NCT> end{};
    CombinedConstantData<NCT> constants{};

    boolean is_private = true; // TODO: might need to instantiate from witness!

    boolean operator==(KernelCircuitPublicInputs<NCT> const& other) const
    {
        return end == other.end && constants == other.constants && is_private == other.is_private;
    };

    template <typename Composer>
    KernelCircuitPublicInputs<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        KernelCircuitPublicInputs<CircuitTypes<Composer>> private_inputs = {
            end.to_circuit_type(composer),
            constants.to_circuit_type(composer),

            to_ct(is_private),
        };

        return private_inputs;
    };

    template <typename Composer> KernelCircuitPublicInputs<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Composer>(); };

        KernelCircuitPublicInputs<NativeTypes> pis = {
            to_native_type(end),
            to_native_type(constants),

            to_nt(is_private),
        };

        return pis;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        end.set_public();
        constants.set_public();

        fr(is_private).set_public();
    }
};

template <typename NCT> void read(uint8_t const*& it, KernelCircuitPublicInputs<NCT>& public_inputs)
{
    using serialize::read;

    read(it, public_inputs.end);
    read(it, public_inputs.constants);
    read(it, public_inputs.is_private);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, KernelCircuitPublicInputs<NCT> const& public_inputs)
{
    using serialize::write;

    write(buf, public_inputs.end);
    write(buf, public_inputs.constants);
    write(buf, public_inputs.is_private);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, KernelCircuitPublicInputs<NCT> const& public_inputs)
{
    return os << "end:\n"
              << public_inputs.end << "\n"
              << "constants:\n"
              << public_inputs.constants << "\n"
              << "is_private: " << public_inputs.is_private << "\n";
}

} // namespace aztec3::circuits::abis