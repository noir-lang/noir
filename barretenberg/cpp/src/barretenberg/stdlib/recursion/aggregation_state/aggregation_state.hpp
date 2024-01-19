#pragma once
#include "../../primitives/field/field.hpp"

namespace bb::plonk {
namespace stdlib {
namespace recursion {

/**
 * Aggregation state contains the following:
 *   (P0, P1): the aggregated elements storing the verification results of proofs in the past
 *   proof_witness_indices: witness indices that point to (P0, P1)
 *   public_inputs: the public inputs of the inner proof, these become the private inputs to the recursive circuit
 *   has_data: indicates if this aggregation state contain past (P0, P1)
 */
template <typename Curve> struct aggregation_state {
    typename Curve::Group P0;
    typename Curve::Group P1;

    // The public inputs of the inner circuit are now private inputs of the outer circuit!
    std::vector<typename Curve::ScalarField> public_inputs;
    std::vector<uint32_t> proof_witness_indices;
    bool has_data = false;

    typename Curve::bool_ct operator==(aggregation_state const& other) const
    {
        return P0 == other.P0 && P1 == other.P1 && public_inputs == other.public_inputs &&
               proof_witness_indices == other.proof_witness_indices;
        //    has_data == other.has_data; can't compare as native
    };

    /**
     * @brief TODO(@dbanks12 please migrate A3 circuits to using `assign_object_to_proof_outputs`. Much safer to not
     * independently track `proof_witness_indices` and whether object has been assigned to public inputs)
     *
     */
    void add_proof_outputs_as_public_inputs()
    {
        auto* context = P0.get_context();
        context->add_recursive_proof(proof_witness_indices);
    }

    void assign_object_to_proof_outputs()
    {
        if (proof_witness_indices.size() == 0) {
            std::cerr << "warning. calling `assign_object_to_proof_outputs`, but aggregation object already has "
                         "assigned proof outputs to public inputs.";
            return;
        }

        P0 = P0.reduce();
        P1 = P1.reduce();
        proof_witness_indices = {
            P0.x.binary_basis_limbs[0].element.normalize().witness_index,
            P0.x.binary_basis_limbs[1].element.normalize().witness_index,
            P0.x.binary_basis_limbs[2].element.normalize().witness_index,
            P0.x.binary_basis_limbs[3].element.normalize().witness_index,
            P0.y.binary_basis_limbs[0].element.normalize().witness_index,
            P0.y.binary_basis_limbs[1].element.normalize().witness_index,
            P0.y.binary_basis_limbs[2].element.normalize().witness_index,
            P0.y.binary_basis_limbs[3].element.normalize().witness_index,
            P1.x.binary_basis_limbs[0].element.normalize().witness_index,
            P1.x.binary_basis_limbs[1].element.normalize().witness_index,
            P1.x.binary_basis_limbs[2].element.normalize().witness_index,
            P1.x.binary_basis_limbs[3].element.normalize().witness_index,
            P1.y.binary_basis_limbs[0].element.normalize().witness_index,
            P1.y.binary_basis_limbs[1].element.normalize().witness_index,
            P1.y.binary_basis_limbs[2].element.normalize().witness_index,
            P1.y.binary_basis_limbs[3].element.normalize().witness_index,
        };

        auto* context = P0.get_context();

        context->check_circuit();
        info("checked circuit before add_recursive_proof");
        context->add_recursive_proof(proof_witness_indices);
    }
};

template <typename Curve> void read(uint8_t const*& it, aggregation_state<Curve>& as)
{
    using serialize::read;

    read(it, as.P0);
    read(it, as.P1);
    read(it, as.public_inputs);
    read(it, as.proof_witness_indices);
    read(it, as.has_data);
};

template <typename Curve> void write(std::vector<uint8_t>& buf, aggregation_state<Curve> const& as)
{
    using serialize::write;

    write(buf, as.P0);
    write(buf, as.P1);
    write(buf, as.public_inputs);
    write(buf, as.proof_witness_indices);
    write(buf, as.has_data);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, aggregation_state<NCT> const& as)
{
    return os << "P0: " << as.P0 << "\n"
              << "P1: " << as.P1 << "\n"
              << "public_inputs: " << as.public_inputs << "\n"
              << "proof_witness_indices: " << as.proof_witness_indices << "\n"
              << "has_data: " << as.has_data << "\n";
}

} // namespace recursion
} // namespace stdlib
} // namespace bb::plonk
