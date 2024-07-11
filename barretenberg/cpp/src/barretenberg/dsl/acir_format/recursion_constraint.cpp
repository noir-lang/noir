#include "recursion_constraint.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"
#include "barretenberg/plonk/transcript/transcript_wrappers.hpp"
#include "barretenberg/stdlib/plonk_recursion/aggregation_state/aggregation_state.hpp"
#include "barretenberg/stdlib/plonk_recursion/verifier/verifier.hpp"
#include "barretenberg/stdlib/primitives/bigfield/constants.hpp"

namespace acir_format {

using namespace bb;

using Transcript_ct = bb::stdlib::recursion::Transcript<Builder>;
using bn254 = stdlib::bn254<Builder>;
using noir_recursive_settings = stdlib::recursion::recursive_ultra_verifier_settings<bn254>;
using verification_key_ct = stdlib::recursion::verification_key<bn254>;
using field_ct = stdlib::field_t<Builder>;
using Composer = plonk::UltraComposer;
using bn254 = stdlib::bn254<Builder>;
using aggregation_state_ct = stdlib::recursion::aggregation_state<bn254>;

using namespace plonk;

// `NUM_LIMB_BITS_IN_FIELD_SIMULATION` is the limb size when simulating a non-native field using the bigfield class
// A aggregation object is two acir_format::g1_ct types where each coordinate in a point is a non-native field.
// Each field is represented as four limbs. We split those limbs in half when serializing to/from buffer.
static constexpr uint64_t TWO_LIMBS_BITS_IN_FIELD_SIMULATION = stdlib::NUM_LIMB_BITS_IN_FIELD_SIMULATION * 2;
static constexpr uint64_t FOUR_LIMBS_BITS_IN_FIELD_SIMULATION = stdlib::NUM_LIMB_BITS_IN_FIELD_SIMULATION * 4;

void generate_dummy_proof() {}
/**
 * @brief Add constraints required to recursively verify an UltraPlonk proof
 *
 * @param builder
 * @param input
 * @tparam has_valid_witness_assignment. Do we have witnesses or are we just generating keys?
 * @tparam inner_proof_contains_recursive_proof. Do we expect the inner proof to also have performed recursive
 * verification? We need to know this at circuit-compile time.
 *
 * @note We currently only support RecursionConstraint where inner_proof_contains_recursive_proof = false.
 *       We would either need a separate ACIR opcode where inner_proof_contains_recursive_proof = true,
 *       or we need non-witness data to be provided as metadata in the ACIR opcode
 */
std::array<uint32_t, RecursionConstraint::AGGREGATION_OBJECT_SIZE> create_recursion_constraints(
    Builder& builder,
    const RecursionConstraint& input,
    const std::array<uint32_t, RecursionConstraint::AGGREGATION_OBJECT_SIZE>& input_aggregation_object,
    const std::array<uint32_t, RecursionConstraint::AGGREGATION_OBJECT_SIZE>& nested_aggregation_object,
    bool has_valid_witness_assignments)
{
    const auto& nested_aggregation_indices = nested_aggregation_object;
    bool nested_aggregation_indices_all_zero = true;
    for (const auto& idx : nested_aggregation_indices) {
        nested_aggregation_indices_all_zero &= (idx == 0);
    }
    const bool inner_proof_contains_recursive_proof = !nested_aggregation_indices_all_zero;

    // If we do not have a witness, we must ensure that our dummy witness will not trigger
    // on-curve errors and inverting-zero errors
    {
        // get a fake key/proof that satisfies on-curve + inversion-zero checks
        const std::vector<fr> dummy_key = export_dummy_key_in_recursion_format(
            PolynomialManifest(Builder::CIRCUIT_TYPE), inner_proof_contains_recursive_proof);
        const auto manifest = Composer::create_manifest(input.public_inputs.size());
        std::vector<fr> dummy_proof =
            export_dummy_transcript_in_recursion_format(manifest, inner_proof_contains_recursive_proof);

        for (size_t i = 0; i < input.public_inputs.size(); ++i) {
            const auto public_input_idx = input.public_inputs[i];
            // if we do NOT have a witness assignment (i.e. are just building the proving/verification keys),
            // we add our dummy public input values as Builder variables.
            // if we DO have a valid witness assignment, we use the real witness assignment
            fr dummy_field = has_valid_witness_assignments ? builder.get_variable(public_input_idx) : dummy_proof[i];
            // Create a copy constraint between our dummy field and the witness index provided by RecursionConstraint.
            // This will make the RecursionConstraint idx equal to `dummy_field`.
            // In the case of a valid witness assignment, this does nothing (as dummy_field = real value)
            // In the case of no valid witness assignment, this makes sure that the RecursionConstraint witness indices
            // will not trigger basic errors (check inputs are on-curve, check we are not inverting 0)
            //
            // Failing to do these copy constraints on public inputs will trigger these basic errors
            // in the case of a nested proof, as an aggregation object is expected to be two G1 points even
            // in the case of no valid witness assignments.
            builder.assert_equal(builder.add_variable(dummy_field), public_input_idx);
        }
        // Remove the public inputs from the dummy proof
        // The proof supplied to the recursion constraint will already be stripped of public inputs
        // while the barretenberg API works with public inputs prepended to the proof.
        dummy_proof.erase(dummy_proof.begin(),
                          dummy_proof.begin() + static_cast<std::ptrdiff_t>(input.public_inputs.size()));
        for (size_t i = 0; i < input.proof.size(); ++i) {
            const auto proof_field_idx = input.proof[i];
            fr dummy_field = has_valid_witness_assignments ? builder.get_variable(proof_field_idx) : dummy_proof[i];
            builder.assert_equal(builder.add_variable(dummy_field), proof_field_idx);
        }
        for (size_t i = 0; i < input.key.size(); ++i) {
            const auto key_field_idx = input.key[i];
            fr dummy_field = has_valid_witness_assignments ? builder.get_variable(key_field_idx) : dummy_key[i];
            builder.assert_equal(builder.add_variable(dummy_field), key_field_idx);
        }
    }

    // Construct an in-circuit representation of the verification key.
    // For now, the v-key is a circuit constant and is fixed for the circuit.
    // (We may need a separate recursion opcode for this to vary, or add more config witnesses to this opcode)
    const auto& aggregation_input = input_aggregation_object;
    aggregation_state_ct previous_aggregation;

    // If we have previously recursively verified proofs, `is_aggregation_object_nonzero = true`
    // For now this is a complile-time constant i.e. whether this is true/false is fixed for the circuit!
    bool inner_aggregation_indices_all_zero = true;
    for (const auto& idx : aggregation_input) {
        inner_aggregation_indices_all_zero &= (idx == 0);
    }

    if (!inner_aggregation_indices_all_zero) {
        std::array<bn254::BaseField, 4> aggregation_elements;
        for (size_t i = 0; i < 4; ++i) {
            aggregation_elements[i] =
                bn254::BaseField(field_ct::from_witness_index(&builder, aggregation_input[4 * i]),
                                 field_ct::from_witness_index(&builder, aggregation_input[4 * i + 1]),
                                 field_ct::from_witness_index(&builder, aggregation_input[4 * i + 2]),
                                 field_ct::from_witness_index(&builder, aggregation_input[4 * i + 3]));
            aggregation_elements[i].assert_is_in_field();
        }
        // If we have a previous aggregation object, assign it to `previous_aggregation` so that it is included
        // in stdlib::recursion::verify_proof
        previous_aggregation.P0 = bn254::Group(aggregation_elements[0], aggregation_elements[1]);
        previous_aggregation.P1 = bn254::Group(aggregation_elements[2], aggregation_elements[3]);
        previous_aggregation.has_data = true;
    } else {
        previous_aggregation.has_data = false;
    }

    transcript::Manifest manifest = Composer::create_manifest(input.public_inputs.size());

    std::vector<field_ct> key_fields;
    key_fields.reserve(input.key.size());
    for (const auto& idx : input.key) {
        auto field = field_ct::from_witness_index(&builder, idx);
        key_fields.emplace_back(field);
    }

    std::vector<field_ct> proof_fields;
    // Prepend the public inputs to the proof fields because this is how the
    // core barretenberg library processes proofs (with the public inputs first and not separated)
    proof_fields.reserve(input.proof.size() + input.public_inputs.size());
    for (const auto& idx : input.public_inputs) {
        auto field = field_ct::from_witness_index(&builder, idx);
        proof_fields.emplace_back(field);
    }
    for (const auto& idx : input.proof) {
        auto field = field_ct::from_witness_index(&builder, idx);
        proof_fields.emplace_back(field);
    }

    // recursively verify the proof
    std::shared_ptr<verification_key_ct> vkey = verification_key_ct::from_field_elements(
        &builder, key_fields, inner_proof_contains_recursive_proof, nested_aggregation_indices);
    vkey->program_width = noir_recursive_settings::program_width;

    Transcript_ct transcript(&builder, manifest, proof_fields, input.public_inputs.size());
    aggregation_state_ct result = stdlib::recursion::verify_proof_<bn254, noir_recursive_settings>(
        &builder, vkey, transcript, previous_aggregation);

    // Assign correct witness value to the verification key hash
    vkey->hash().assert_equal(field_ct::from_witness_index(&builder, input.key_hash));

    ASSERT(result.public_inputs.size() == input.public_inputs.size());

    // Assign the `public_input` field to the public input of the inner proof
    for (size_t i = 0; i < input.public_inputs.size(); ++i) {
        result.public_inputs[i].assert_equal(field_ct::from_witness_index(&builder, input.public_inputs[i]));
    }

    // We want to return an array, so just copy the vector into the array
    ASSERT(result.proof_witness_indices.size() == RecursionConstraint::AGGREGATION_OBJECT_SIZE);
    std::array<uint32_t, RecursionConstraint::AGGREGATION_OBJECT_SIZE> resulting_output_aggregation_object;
    std::copy(result.proof_witness_indices.begin(),
              result.proof_witness_indices.begin() + RecursionConstraint::AGGREGATION_OBJECT_SIZE,
              resulting_output_aggregation_object.begin());

    return resulting_output_aggregation_object;
}

/**
 * @brief When recursively verifying proofs, we represent the verification key using field elements.
 *        This method exports the key formatted in the manner our recursive verifier expects.
 *        NOTE: only used by the dsl at the moment. Might be cleaner to make this a dsl function?
 *
 * @return std::vector<fr>
 */
std::vector<fr> export_key_in_recursion_format(std::shared_ptr<verification_key> const& vkey)
{
    std::vector<fr> output;
    output.emplace_back(vkey->domain.root);
    output.emplace_back(vkey->domain.domain);
    output.emplace_back(vkey->domain.generator);
    output.emplace_back(vkey->circuit_size);
    output.emplace_back(vkey->num_public_inputs);
    output.emplace_back(vkey->contains_recursive_proof);
    for (size_t i = 0; i < RecursionConstraint::AGGREGATION_OBJECT_SIZE; ++i) {
        if (vkey->recursive_proof_public_input_indices.size() > i) {
            output.emplace_back(vkey->recursive_proof_public_input_indices[i]);
        } else {
            output.emplace_back(0);
            ASSERT(vkey->contains_recursive_proof == false);
        }
    }
    for (const auto& descriptor : vkey->polynomial_manifest.get()) {
        if (descriptor.source == PolynomialSource::SELECTOR || descriptor.source == PolynomialSource::PERMUTATION) {
            const auto element = vkey->commitments.at(std::string(descriptor.commitment_label));
            auto g1_as_fields = export_g1_affine_element_as_fields(element);
            output.emplace_back(g1_as_fields.x_lo);
            output.emplace_back(g1_as_fields.x_hi);
            output.emplace_back(g1_as_fields.y_lo);
            output.emplace_back(g1_as_fields.y_hi);
        }
    }

    verification_key_data vkey_data{
        .circuit_type = static_cast<uint32_t>(vkey->circuit_type),
        .circuit_size = static_cast<uint32_t>(vkey->circuit_size),
        .num_public_inputs = static_cast<uint32_t>(vkey->num_public_inputs),
        .commitments = vkey->commitments,
        .contains_recursive_proof = vkey->contains_recursive_proof,
        .recursive_proof_public_input_indices = vkey->recursive_proof_public_input_indices,
    };
    output.emplace_back(vkey_data.hash_native(0)); // key_hash
    return output;
}

/**
 * @brief When recursively verifying proofs, we represent the verification key using field elements.
 *        This method exports the key formatted in the manner our recursive verifier expects.
 *        A dummy key is used when building a circuit without a valid witness assignment.
 *        We want the transcript to contain valid G1 points to prevent on-curve errors being thrown.
 *        We want a non-zero circuit size as this element will be inverted by the circuit
 *        and we do not want an "inverting 0" error thrown
 *
 * @return std::vector<fr>
 */
std::vector<fr> export_dummy_key_in_recursion_format(const PolynomialManifest& polynomial_manifest,
                                                     const bool contains_recursive_proof)
{
    std::vector<fr> output;
    output.emplace_back(1); // domain.domain (will be inverted)
    output.emplace_back(1); // domain.root (will be inverted)
    output.emplace_back(1); // domain.generator (will be inverted)

    output.emplace_back(1); // circuit size
    output.emplace_back(1); // num public inputs

    output.emplace_back(contains_recursive_proof); // contains_recursive_proof
    for (size_t i = 0; i < RecursionConstraint::AGGREGATION_OBJECT_SIZE; ++i) {
        output.emplace_back(0); // recursive_proof_public_input_indices
    }

    for (const auto& descriptor : polynomial_manifest.get()) {
        if (descriptor.source == PolynomialSource::SELECTOR || descriptor.source == PolynomialSource::PERMUTATION) {
            // the std::biggroup class creates unsatisfiable constraints when identical points are added/subtracted.
            // (when verifying zk proofs this is acceptable as we make sure verification key points are not identical.
            // And prover points should contain randomness for an honest Prover).
            // This check can also trigger a runtime error due to causing 0 to be inverted.
            // When creating dummy verification key points we must be mindful of the above and make sure that each
            // transcript point is unique.
            auto scalar = fr::random_element();
            const auto element = g1::affine_element(g1::one * scalar);
            auto g1_as_fields = export_g1_affine_element_as_fields(element);
            output.emplace_back(g1_as_fields.x_lo);
            output.emplace_back(g1_as_fields.x_hi);
            output.emplace_back(g1_as_fields.y_lo);
            output.emplace_back(g1_as_fields.y_hi);
        }
    }

    output.emplace_back(0); // key_hash

    return output;
}

/**
 * @brief Returns transcript represented as a vector of fr.
 *        Used to represent recursive proofs (i.e. proof represented as circuit-native field elements)
 *
 * @return std::vector<fr>
 */
std::vector<fr> export_transcript_in_recursion_format(const transcript::StandardTranscript& transcript)
{
    std::vector<fr> fields;
    const auto num_rounds = transcript.get_manifest().get_num_rounds();
    for (size_t i = 0; i < num_rounds; ++i) {
        for (const auto& manifest_element : transcript.get_manifest().get_round_manifest(i).elements) {
            if (!manifest_element.derived_by_verifier) {
                if (manifest_element.num_bytes == 32 && manifest_element.name != "public_inputs") {
                    fields.emplace_back(transcript.get_field_element(manifest_element.name));
                } else if (manifest_element.num_bytes == 64 && manifest_element.name != "public_inputs") {
                    const auto group_element = transcript.get_group_element(manifest_element.name);
                    auto g1_as_fields = export_g1_affine_element_as_fields(group_element);
                    fields.emplace_back(g1_as_fields.x_lo);
                    fields.emplace_back(g1_as_fields.x_hi);
                    fields.emplace_back(g1_as_fields.y_lo);
                    fields.emplace_back(g1_as_fields.y_hi);
                } else {
                    ASSERT(manifest_element.name == "public_inputs");
                    const auto public_inputs_vector = transcript.get_field_element_vector(manifest_element.name);
                    for (const auto& ele : public_inputs_vector) {
                        fields.emplace_back(ele);
                    }
                }
            }
        }
    }
    return fields;
}

/**
 * @brief Get a dummy fake proof for recursion. All elliptic curve group elements are still valid points to prevent
 * errors being thrown.
 *
 * @param manifest
 * @return std::vector<fr>
 */
std::vector<fr> export_dummy_transcript_in_recursion_format(const transcript::Manifest& manifest,
                                                            const bool contains_recursive_proof)
{
    std::vector<fr> fields;
    const auto num_rounds = manifest.get_num_rounds();
    for (size_t i = 0; i < num_rounds; ++i) {
        for (const auto& manifest_element : manifest.get_round_manifest(i).elements) {
            if (!manifest_element.derived_by_verifier) {
                if (manifest_element.num_bytes == 32 && manifest_element.name != "public_inputs") {
                    // auto scalar = fr::random_element();
                    fields.emplace_back(0);
                } else if (manifest_element.num_bytes == 64 && manifest_element.name != "public_inputs") {
                    // the std::biggroup class creates unsatisfiable constraints when identical points are
                    // added/subtracted.
                    // (when verifying zk proofs this is acceptable as we make sure verification key points are not
                    // identical. And prover points should contain randomness for an honest Prover). This check can
                    // also trigger a runtime error due to causing 0 to be inverted. When creating dummy proof
                    // points we must be mindful of the above and make sure that each point is unique.
                    auto scalar = fr::random_element();
                    const auto group_element = g1::affine_element(g1::one * scalar);
                    auto g1_as_fields = export_g1_affine_element_as_fields(group_element);
                    fields.emplace_back(g1_as_fields.x_lo);
                    fields.emplace_back(g1_as_fields.x_hi);
                    fields.emplace_back(g1_as_fields.y_lo);
                    fields.emplace_back(g1_as_fields.y_hi);
                } else {
                    ASSERT(manifest_element.name == "public_inputs");
                    const size_t num_public_inputs = manifest_element.num_bytes / 32;
                    // If we have a recursive proofs the public inputs must describe an aggregation object that
                    // is composed of two valid G1 points on the curve. Without this conditional we will get a
                    // runtime error that we are attempting to invert 0.
                    if (contains_recursive_proof) {
                        // When setting up the ACIR we emplace back the nested aggregation object
                        // fetched from the proof onto the public inputs. Thus, we can expect the
                        // nested aggregation object to always be at the end of the public inputs.
                        for (size_t k = 0; k < num_public_inputs - RecursionConstraint::AGGREGATION_OBJECT_SIZE; ++k) {
                            fields.emplace_back(0);
                        }
                        for (size_t k = 0; k < RecursionConstraint::NUM_AGGREGATION_ELEMENTS; ++k) {
                            auto scalar = fr::random_element();
                            const auto group_element = g1::affine_element(g1::one * scalar);
                            auto g1_as_fields = export_g1_affine_element_as_fields(group_element);
                            fields.emplace_back(g1_as_fields.x_lo);
                            fields.emplace_back(g1_as_fields.x_hi);
                            fields.emplace_back(g1_as_fields.y_lo);
                            fields.emplace_back(g1_as_fields.y_hi);
                        }
                    } else {
                        for (size_t j = 0; j < num_public_inputs; ++j) {
                            // auto scalar = fr::random_element();
                            fields.emplace_back(0);
                        }
                    }
                }
            }
        }
    }
    return fields;
}

size_t recursion_proof_size_without_public_inputs()
{
    const auto manifest = Composer::create_manifest(0);
    auto dummy_transcript = export_dummy_transcript_in_recursion_format(manifest, false);
    return dummy_transcript.size();
}

G1AsFields export_g1_affine_element_as_fields(const g1::affine_element& group_element)
{
    const uint256_t x = group_element.x;
    const uint256_t y = group_element.y;
    const fr x_lo = x.slice(0, TWO_LIMBS_BITS_IN_FIELD_SIMULATION);
    const fr x_hi = x.slice(TWO_LIMBS_BITS_IN_FIELD_SIMULATION, FOUR_LIMBS_BITS_IN_FIELD_SIMULATION);
    const fr y_lo = y.slice(0, TWO_LIMBS_BITS_IN_FIELD_SIMULATION);
    const fr y_hi = y.slice(TWO_LIMBS_BITS_IN_FIELD_SIMULATION, FOUR_LIMBS_BITS_IN_FIELD_SIMULATION);

    return G1AsFields{ x_lo, x_hi, y_lo, y_hi };
}

} // namespace acir_format
