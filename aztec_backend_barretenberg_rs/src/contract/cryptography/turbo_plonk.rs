
#[macro_export]
macro_rules! TURBOPLONK_LIBRARY {
    () => { r#"
    /**
     * @title TurboPlonk verification algo implementation
     * @dev Implements the Turbo Plonk verification algorithm, through the use of five functions that
     * calculate challenges, setup initial state, evaluate the necessary polynomials and executes the pairing
     * check.
     *
     * Expected to be inherited by `Verifier.sol`
     *
     * Copyright 2020 Spilsbury Holdings Ltd
     *
     * Licensed under the GNU General Public License, Version 2.0 (the "License");
     * you may not use this file except in compliance with the License.
     *
     * This program is distributed in the hope that it will be useful,
     * but WITHOUT ANY WARRANTY; without even the implied warranty of
     * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
     * GNU General Public License for more details.
     *
     * You should have received a copy of the GNU General Public License
     * along with this program.  If not, see <https://www.gnu.org/licenses/>.
     */
    library TurboPlonk {
        using PairingsBn254 for Types.G1Point;
        using PairingsBn254 for Types.G2Point;
        using PairingsBn254 for Types.Fr;
        using TranscriptLibrary for TranscriptLibrary.Transcript;
    
        /**
         * @dev Evaluate the various remaining polynomials: partial_opening, batch_opening, batch_evaluation
         * @param decoded_proof - deserialised proof
         * @param vk - verification key
         * @param challenges - all challenges (alpha, beta, gamma, zeta, nu[NUM_NU_CHALLENGES], u) stored in
         * ChallengeTranscript struct form
         * @param L1 - lagrange 1 evaluation
         * @return batch_opening commitment and batch_evaluation commitment, both represented as G1 Points
         */
        function evaluate_polynomials(
            Types.Proof memory decoded_proof,
            Types.VerificationKey memory vk,
            Types.ChallengeTranscript memory challenges,
            Types.Fr memory L1
        ) internal view returns (Types.G1Point memory, Types.G1Point memory) {
            Types.G1Point memory partial_opening_commitment = PolynomialEval.compute_partial_opening_commitment(
                challenges,
                L1,
                vk.sigma_commitments[2],
                vk,
                decoded_proof
            );
    
            Types.G1Point memory batch_opening_commitment = PolynomialEval.compute_batch_opening_commitment(
                challenges,
                vk,
                partial_opening_commitment,
                decoded_proof
            );
    
            (Types.G1Point memory batch_evaluation_commitment, ) = PolynomialEval.compute_batch_evaluation_commitment(
                decoded_proof,
                challenges
            );
    
            return (batch_opening_commitment, batch_evaluation_commitment);
        }
    
        /**
         * @dev Compute partial state of the verifier, specifically: public input delta evaluation, zero polynomial
         * evaluation, the lagrange evaluations and the quotient polynomial evaluations
         *
         * Note: This uses the batch inversion Montgomery trick to reduce the number of
         * inversions, and therefore the number of calls to the bn128 modular exponentiation
         * precompile.
         *
         * Specifically, each function call: compute_public_input_delta() etc. at some point needs to invert a
         * value to calculate a denominator in a fraction. Instead of performing this inversion as it is needed, we
         * instead 'save up' the denominator calculations. The inputs to this are returned from the various functions
         * and then we perform all necessary inversions in one go at the end of `compute_partial_state()`. This
         * gives us the various variables that need to be returned.
         *
         * It should be noted that an intermediate inverse is
         *
         * @param decoded_proof - deserialised proof
         * @param vk - verification key
         * @param challenges - all challenges (alpha, beta, gamma, zeta, nu[NUM_NU_CHALLENGES], u) stored in
         * ChallengeTranscript struct form
         * @return quotient polynomial evaluation (field element) and lagrange 1 evaluation (field element)
         */
        function compute_partial_state(
            Types.Proof memory decoded_proof,
            Types.VerificationKey memory vk,
            Types.ChallengeTranscript memory challenges
        ) internal view returns (Types.Fr memory, Types.Fr memory) {
            Types.Fr memory public_input_delta;
            Types.Fr memory zero_polynomial_eval;
            Types.Fr[] memory lagrange_evals;
            {
                Types.PartialStateFractions memory partial_state_fractions;
    
                Types.Fraction memory public_input_delta_fraction = PolynomialEval.compute_public_input_delta(
                    decoded_proof.public_input_values,
                    challenges,
                    vk
                );
                partial_state_fractions.public_input_delta = public_input_delta_fraction;
    
                Types.Fraction memory zero_poly_fraction = PolynomialEval.compute_zero_polynomial(
                    challenges.zeta,
                    vk.circuit_size,
                    vk.work_root_inverse
                );
                partial_state_fractions.zero_poly = zero_poly_fraction;
    
                // lagrange_evals[0] = L1, lagrange_evals[1] = Ln
    
                Types.Fraction[] memory lagrange_eval_fractions = PolynomialEval.compute_lagrange_evaluations(
                    vk,
                    challenges.zeta
                );
                partial_state_fractions.lagrange_1_fraction = lagrange_eval_fractions[0];
                partial_state_fractions.lagrange_n_fraction = lagrange_eval_fractions[1];
    
                (zero_polynomial_eval, public_input_delta, lagrange_evals) = PolynomialEval.compute_batch_inversions(
                    partial_state_fractions
                );
            }
    
            Types.Fr memory quotient_eval;
            {
                // scope to avoid stack too deep
                quotient_eval = PolynomialEval.compute_quotient_polynomial(
                    zero_polynomial_eval,
                    public_input_delta,
                    challenges,
                    lagrange_evals,
                    decoded_proof
                );
            }
    
            return (quotient_eval, lagrange_evals[0]);
        }
    
        /**
         * @dev Perform the pairing check
         * @param batch_opening_commitment - G1 point representing the calculated batch opening commitment
         * @param batch_evaluation_commitment - G1 point representing the calculated batch evaluation commitment
         * @param challenges - all challenges (alpha, beta, gamma, zeta, nu[NUM_NU_CHALLENGES], u) stored in
         * ChallengeTranscript struct form
         * @param vk - verification key
         * @param decoded_proof - deserialised proof
         * @return bool specifying whether the pairing check was successful
         */
        function perform_pairing(
            Types.G1Point memory batch_opening_commitment,
            Types.G1Point memory batch_evaluation_commitment,
            Types.ChallengeTranscript memory challenges,
            Types.Proof memory decoded_proof,
            Types.VerificationKey memory vk
        ) internal view returns (bool) {
            // lhs
            Types.G1Point memory lhsTerm1 = PairingsBn254.point_add(
                decoded_proof.opening_at_z_proof,
                PairingsBn254.point_mul(decoded_proof.opening_at_z_omega_proof, challenges.u)
            );
    
            lhsTerm1.negate();
    
            // rhs
            // first term
            Types.G1Point memory first_term = PairingsBn254.point_mul(decoded_proof.opening_at_z_proof, challenges.zeta);
    
            // second term
            Types.Fr memory scalars = PairingsBn254.mul_fr(
                challenges.u,
                (PairingsBn254.mul_fr(challenges.zeta, vk.work_root))
            );
            Types.G1Point memory second_term = PairingsBn254.point_mul(decoded_proof.opening_at_z_omega_proof, scalars);
    
            Types.G1Point memory rhsTerm1 = PairingsBn254.point_add(first_term, second_term);
            rhsTerm1.point_add_assign(batch_opening_commitment);
            rhsTerm1.point_sub_assign(batch_evaluation_commitment);
    
            return PairingsBn254.pairingProd2(rhsTerm1, PairingsBn254.P2(), lhsTerm1, vk.g2_x);
        }
    
        /**
         * @dev Calculate the alpha, beta, gamma and zeta challenges
         * Makes use of the Transcript library
         * @param decoded_proof - deserialised proof
         * @param vk - verification key
         * @return challenge transcript containing alpha, beta, gamma, zeta and seperately the
         * general helper transcript containing the data necessary future challenges
         */
        function construct_alpha_beta_gamma_zeta_challenges(
            Types.Proof memory decoded_proof,
            Types.VerificationKey memory vk
        ) internal pure returns (Types.ChallengeTranscript memory, TranscriptLibrary.Transcript memory) {
            // TODO: do these need acting on?
            // require(decoded_proof.public_input_values.length == vk.num_inputs);
            // require(vk.num_inputs >= 1);
            TranscriptLibrary.Transcript memory transcript = TranscriptLibrary.new_transcript(
                vk.circuit_size,
                vk.num_inputs
            );
    
            Types.ChallengeTranscript memory challenges;
    
            challenges.init = Types.Fr({value: uint256(transcript.current_challenge) % Types.r_mod});
    
            for (uint256 i = 0; i < vk.num_inputs; i++) {
                transcript.update_with_u256(decoded_proof.public_input_values[i]);
            }
            assert(decoded_proof.wire_commitments.length == 4);
            for (uint256 i = 0; i < decoded_proof.wire_commitments.length; i++) {
                transcript.update_with_g1(decoded_proof.wire_commitments[i]);
            }
            challenges.debug_data = transcript.debug_data;
            challenges.beta = transcript.get_challenge();
    
            transcript.append_byte(1);
            challenges.gamma = transcript.get_challenge();
    
            transcript.update_with_g1(decoded_proof.grand_product_commitment);
            challenges.alpha = transcript.get_challenge();
            challenges.alpha_base = PairingsBn254.new_fr(challenges.alpha.value);
    
            for (uint256 i = 0; i < Types.STATE_WIDTH; i += 1) {
                transcript.update_with_g1(decoded_proof.quotient_poly_commitments[i]);
            }
            challenges.zeta = transcript.get_challenge();
    
            return (challenges, transcript);
        }
    
        /**
         * @dev Calculate the remaining challenges: nu[] and u.
         * For Turbo PLONK, 11 challenges are calculated. Makes use of the Transcript library
         *
         * @param decoded_proof - deserialised proof
         * @param transcript - general transcript containing the data necessary to calculate subsequent
         * challenges
         * @param challenges - challenge transcript containing alpha, beta, gamma, zeta
         * @return challenge transcript containing the original challenges, together with the calculated
         * nu's and u challenges
         */
        function construct_nu_u_challenges(
            Types.Proof memory decoded_proof,
            TranscriptLibrary.Transcript memory transcript,
            Types.ChallengeTranscript memory challenges
        ) internal pure returns (Types.ChallengeTranscript memory) {
            transcript.update_with_fr(decoded_proof.quotient_polynomial_at_z);
    
            for (uint256 i = 0; i < Types.STATE_WIDTH; i++) {
                transcript.update_with_fr(decoded_proof.wire_values_at_z[i]);
            }
    
            for (uint256 i = 0; i < decoded_proof.permutation_polynomials_at_z.length; i++) {
                transcript.update_with_fr(decoded_proof.permutation_polynomials_at_z[i]);
            }
    
            transcript.update_with_fr(decoded_proof.q_arith_at_z);
            transcript.update_with_fr(decoded_proof.q_ecc_at_z);
            transcript.update_with_fr(decoded_proof.q_c_at_z);
    
            transcript.update_with_fr(decoded_proof.linearization_polynomial_at_z);
            transcript.update_with_fr(decoded_proof.grand_product_at_z_omega);
            for (uint256 i = 0; i < Types.STATE_WIDTH; i++) {
                transcript.update_with_fr(decoded_proof.wire_values_at_z_omega[i]);
            }
    
            Types.Fr memory base_v_challenge = transcript.get_challenge();
            bytes32 base_v_challenge_unreduced = transcript.current_challenge;
            challenges.v[0] = base_v_challenge;
    
            /**
             * Aim here is to generate a number of challenges, derived from a baseHash/challenge.
             *
             * To do this, we take the baseHash and then append a byte to the end, which increases in
             * value on each round of the for loop i.e.:
             *
             * challenge1 = keccak256(${baseHash}'01')
             * challenge2 = keccak256(${baseHash}'02')
             *
             */
            for (uint256 i = 1; i < Types.NUM_NU_CHALLENGES; i += 1) {
                // reset to base_v_challenge, and generate next from that
                transcript.data = abi.encodePacked(base_v_challenge_unreduced);
                transcript.append_byte(uint8(i));
                challenges.v[i] = transcript.get_challenge();
            }
    
            transcript.update_with_g1(decoded_proof.opening_at_z_proof);
            transcript.update_with_g1(decoded_proof.opening_at_z_omega_proof);
            challenges.u = transcript.get_challenge();
    
            return challenges;
        }
            /**
         * @dev Deserialize a proof into a Proof struct
         * @param raw_data - raw byte array containing proof data
         * @param num_public_inputs - number of public inputs in the proof. Taken from verification key
         * @return proof - proof deserialized into the proof struct
         */
        function deserialize_proof(bytes memory raw_data, uint256 num_public_inputs)
            internal
            pure
            returns (Types.Proof memory proof)
        {
            uint256 data_ptr;
            uint256 x;
            uint256 y;
            // first 32 bytes of bytes array contains length, skip it
            assembly {
                data_ptr := add(raw_data, 0x20)
            }
    
            proof.public_input_values = new uint256[](num_public_inputs);
    
            for (uint256 i = 0; i < num_public_inputs; ++i) {
                assembly {
                    x := mload(data_ptr)
                }
                proof.public_input_values[i] = x;
                data_ptr += 0x20;
            }
    
            for (uint256 i = 0; i < Types.STATE_WIDTH; ++i) {
                assembly {
                    y := mload(data_ptr)
                    x := mload(add(data_ptr, 0x20))
                }
                proof.wire_commitments[i] = PairingsBn254.new_g1(x, y);
                data_ptr += 0x40;
            }
    
            assembly {
                y := mload(data_ptr)
                x := mload(add(data_ptr, 0x20))
            }
            proof.grand_product_commitment = PairingsBn254.new_g1(x, y);
            data_ptr += 0x40;
    
            for (uint256 i = 0; i < Types.STATE_WIDTH; ++i) {
                assembly {
                    y := mload(data_ptr)
                    x := mload(add(data_ptr, 0x20))
                }
                proof.quotient_poly_commitments[i] = PairingsBn254.new_g1(x, y);
                data_ptr += 0x40;
            }
    
            for (uint256 i = 0; i < Types.STATE_WIDTH; ++i) {
                assembly {
                    x := mload(data_ptr)
                }
                proof.wire_values_at_z[i] = PairingsBn254.new_fr(x);
                data_ptr += 0x20;
            }
    
            for (uint256 i = 0; i < Types.STATE_WIDTH - 1; ++i) {
                assembly {
                    x := mload(data_ptr)
                }
                proof.permutation_polynomials_at_z[i] = PairingsBn254.new_fr(x);
                data_ptr += 0x20;
            }
    
            assembly {
                x := mload(data_ptr)
            }
            proof.q_arith_at_z = PairingsBn254.new_fr(x);
            data_ptr += 0x20;
            assembly {
                x := mload(data_ptr)
            }
            proof.q_ecc_at_z = PairingsBn254.new_fr(x);
            data_ptr += 0x20;
    
            assembly {
                x := mload(data_ptr)
            }
            proof.q_c_at_z = PairingsBn254.new_fr(x);
            data_ptr += 0x20;
    
            assembly {
                x := mload(data_ptr)
            }
            proof.linearization_polynomial_at_z = PairingsBn254.new_fr(x);
            data_ptr += 0x20;
    
            assembly {
                x := mload(data_ptr)
            }
            proof.grand_product_at_z_omega = PairingsBn254.new_fr(x);
            data_ptr += 0x20;
    
            for (uint256 i = 0; i < Types.STATE_WIDTH; ++i) {
                assembly {
                    x := mload(data_ptr)
                }
                proof.wire_values_at_z_omega[i] = PairingsBn254.new_fr(x);
                data_ptr += 0x20;
            }
    
            assembly {
                y := mload(data_ptr)
                x := mload(add(data_ptr, 0x20))
            }
            proof.opening_at_z_proof = PairingsBn254.new_g1(x, y);
            data_ptr += 0x40;
            assembly {
                y := mload(data_ptr)
                x := mload(add(data_ptr, 0x20))
            }
            proof.opening_at_z_omega_proof = PairingsBn254.new_g1(x, y);
            data_ptr += 0x40;
        }
    }
    
    
    "# };
}