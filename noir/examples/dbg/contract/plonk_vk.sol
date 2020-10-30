
// SPDX-License-Identifier: GPL-2.0-only
// Copyright 2020 Spilsbury Holdings Ltd

pragma solidity >=0.6.0 <0.7.0;
pragma experimental ABIEncoderV2;

/**
 * @title Plonk proof verification contract
 * @dev Top level Plonk proof verification contract, which allows Plonk proof to be verified
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
contract TurboVerifier {
    using PairingsBn254 for Types.G1Point;
    using PairingsBn254 for Types.G2Point;
    using PairingsBn254 for Types.Fr;

    /**
     * @dev Verify a Plonk proof
     * @param serialized_proof - array of serialized proof data
     */
    function verify(bytes memory serialized_proof) public view returns (bool) {
        Types.VerificationKey memory vk = get_verification_key();
        uint256 num_public_inputs = vk.num_inputs;

        Types.Proof memory decoded_proof = TurboPlonk.deserialize_proof(serialized_proof, num_public_inputs);
        (Types.ChallengeTranscript memory challenges, TranscriptLibrary.Transcript memory transcript) = TurboPlonk
            .construct_alpha_beta_gamma_zeta_challenges(decoded_proof, vk);

        /**
         * Compute all inverses that will be needed throughout the program here.
         *
         * This is an efficiency improvement - it allows us to make use of the batch inversion Montgomery trick,
         * which allows all inversions to be replaced with one inversion operation, at the expense of a few
         * additional multiplications
         **/
        (Types.Fr memory quotient_eval, Types.Fr memory L1) = TurboPlonk.compute_partial_state(
            decoded_proof,
            vk,
            challenges
        );
        decoded_proof.quotient_polynomial_at_z = PairingsBn254.new_fr(quotient_eval.value);

        //reset 'alpha base'
        challenges = TurboPlonk.construct_nu_u_challenges(decoded_proof, transcript, challenges);
        challenges.alpha_base = PairingsBn254.new_fr(challenges.alpha.value);

        (Types.G1Point memory batch_opening_commitment, Types.G1Point memory batch_evaluation_commitment) = TurboPlonk
            .evaluate_polynomials(decoded_proof, vk, challenges, L1);

        bool result = TurboPlonk.perform_pairing(
            batch_opening_commitment,
            batch_evaluation_commitment,
            challenges,
            decoded_proof,
            vk
        );
        return result;
    }

    
  function get_verification_key() internal pure returns (Types.VerificationKey memory) {
    Types.VerificationKey memory vk;

    vk.circuit_size = 16384;
    vk.num_inputs = 0;
    vk.work_root = PairingsBn254.new_fr(
      0x2d965651cdd9e4811f4e51b80ddca8a8b4a93ee17420aae6adaa01c2617c6e85
    );
    vk.domain_inverse = PairingsBn254.new_fr(
      0x30638ce1a7661b6337a964756aa75257c6bf4778d89789ab819ce60c19b04001
    );
    vk.work_root_inverse = PairingsBn254.new_fr(
      0x281c036f06e7e9e911680d42558e6e8cf40976b0677771c0f8eee934641c8410
    );
    vk.Q1 = PairingsBn254.new_g1(
      0x0d578028329fbb3816547151fa77a1bcab1ad76956a5275ec84cefe0c26ca406,
      0x2361fbc2538ea8f34a2f448831d21947f09de9850c666d70c1ffdcd6fa6c4117
    );
    vk.Q2 = PairingsBn254.new_g1(
      0x1b9305fbf3252aa3ae26800744b29096f40ebdecaf9ef143351232bdb77355ca,
      0x2edc48f5f933f59cc62fede9070db6633063aa0cefdfe2665f6885495022d6fd
    );
    vk.Q3 = PairingsBn254.new_g1(
      0x02fcfe311831689eb300272857dd8cc712841f14b128b3f2aafa1d8492be0996,
      0x17fec35d2b7c743a03e2d043414de07fb45c23082766a85a14c8632373f5fd65
    );
    vk.Q4 = PairingsBn254.new_g1(
      0x3047a7aacc80ab1bccb1308cc5d19959772c95b56ffbeab081bba5dbc002e4a6,
      0x2b5a6e8aeda6023cada9572f24105f952d113bb88d5d7cc864ca7d274614692e
    );
    vk.Q5 = PairingsBn254.new_g1(
      0x15aeede8dedb1634b18657e2dd6044787b3bcc9c25d5cded3a1f78255d1ff59f,
      0x2465db756f2d6c0d2f5b44fb811c8222893253d78f9cd75a479b6ed327c3d28f
    );
    vk.QM = PairingsBn254.new_g1(
      0x2207a2e003b922ed9d5a4e83a1093f01efffe2ceff948dafc272a3b5ced3d02d,
      0x1a5a897a6b2cb18ee930ea891f854e32e9bdd98137d533ef8d184bde0a3ce94f
    );
    vk.QC = PairingsBn254.new_g1(
      0x03bfaf20c545cedb0b95a04783dfbfe475da59edad76c373abf8f8621e20f2d9,
      0x1057bae523171ce7501d65eb4c32c6c9f3430172f6db419688e1d0f904a4ca31
    );
    vk.QARITH = PairingsBn254.new_g1(
      0x18ffa930431631459d48be1e4a99793d22a493a797c9925ecc4be53f594339b4,
      0x01b2675ecbcc1039335dcc1e952e4a50d93f2aa8ad9b660772d99e1b48f74669
    );
    vk.QECC = PairingsBn254.new_g1(
      0x18c2ad6a07192f135fd3a4cd94fb91e6097f90e9ea1c4f8f4743f6547e37c71a,
      0x1ed7587cf8b1039baa690dc832f954b53b1200d320346660d01fc2270f5aaaf7
    );
    vk.QRANGE = PairingsBn254.new_g1(
      0x16ea5b9bc87dd742e1ef79ef1ebd13d0365169b47f45209a8e27195b64663c3e,
      0x1505a43e2919d52478479430187f18b4b16ff86b23dc5a0bd336fe41e6744919
    );
    vk.QLOGIC = PairingsBn254.new_g1(
      0x1c364a41ef76c2b040dea0bee6414afb6c4b91ba49330f40ec273743248ea459,
      0x01e815ef07bb3d32ec6f45ce7c72810f64c3d82728b24b89fc2895ec69bebe68
    );
    vk.sigma_commitments[0] = PairingsBn254.new_g1(
      0x275be82c2e081aca1728804ac7fc5c09bf42016a14b7137f701450c2ee114c33,
      0x04df8e40658b3a18475642fbb509aa80428ed0250487c0beb6b2a5f66a81b4fd
    );
    vk.sigma_commitments[1] = PairingsBn254.new_g1(
      0x11089219d201aa0ff49915fe7d1dc31b499aa2b087a3596c6b4b03292dea4b7e,
      0x0eafdfda05ca454e0e8a35d726d5990c43685f1ced8fc145a6956e2a4b3bb949
    );
    vk.sigma_commitments[2] = PairingsBn254.new_g1(
      0x27008cdc4ecbf00a2ed47978e4ee48b2ef8a09237b0a0bddceba7845f71b69a8,
      0x2ccab05f8633d800479768338045e7648887eff148ee456aa307d52095c6a13a
    );
    vk.sigma_commitments[3] = PairingsBn254.new_g1(
      0x2fca0830423e97a85dc1f4f252e4e5bbbcd32d3e305c8da70c7e9a2286c67b8b,
      0x16f7892e9f774f8680f6c104a2fa625eba8b9bc63369fc4d0458312175a7792e
    );
    vk.permutation_non_residues[0] = PairingsBn254.new_fr(
      0x0000000000000000000000000000000000000000000000000000000000000005
    );
    vk.permutation_non_residues[1] = PairingsBn254.new_fr(
      0x0000000000000000000000000000000000000000000000000000000000000006
    );
    vk.permutation_non_residues[2] = PairingsBn254.new_fr(
      0x0000000000000000000000000000000000000000000000000000000000000007
    );

    vk.g2_x = PairingsBn254.new_g2([
      0x260e01b251f6f1c7e7ff4e580791dee8ea51d87a358e038b4efe30fac09383c1,
      0x0118c4d5b837bcc2bc89b5b398b5974e9f5944073b32078b7e231fec938883b0
    ],[
      0x04fc6369f7110fe3d25156c1bb9a72859cf2a04641f99ba4ee413c80da6a5fe4,
      0x22febda3c0c0632a56475b4214e5615e11e6dd3f96e6cea2854a87d4dacc5e55
    ]);
    return vk;
  }
}



}


/**
 * @title PairingsBn254 library used for the fr, g1 and g2 point types
 * @dev Used to manipulate fr, g1, g2 types, perform modular arithmetic on them and call
 * the precompiles add, scalar mul and pairing
 *
 * Notes on optimisations
 * 1) Perform addmod, mulmod etc. in assembly - removes the check that Solidity performs to confirm that
 * the supplied modulus is not 0. This is safe as the modulus's used (r_mod, q_mod) are hard coded
 * inside the contract and not supplied by the user
 */
library Types {
    uint256 constant STATE_WIDTH = 4;
    uint256 constant NUM_NU_CHALLENGES = 11;
    uint256 constant PRIM_ROOT_SIZE = 28;
    uint256 constant NUM_KATE_OPENING_ELEMENTS = 28; // TODO check this, could be smaller
    uint256 constant PRIM_ROOT = 0x2a3c09f0a58a7e8500e0a7eb8ef62abc402d111e41112ed49bd61b6e725b19f0;
    uint256 constant r_mod = 21888242871839275222246405745257275088548364400416034343698204186575808495617;

    uint256 constant coset_generator0 = 0x0000000000000000000000000000000000000000000000000000000000000005;
    uint256 constant coset_generator1 = 0x0000000000000000000000000000000000000000000000000000000000000006;
    uint256 constant coset_generator2 = 0x0000000000000000000000000000000000000000000000000000000000000007;

    // TODO: add external_coset_generator() method to compute this
    uint256 constant coset_generator7 = 0x000000000000000000000000000000000000000000000000000000000000000c;

    struct G1Point {
        uint256 X;
        uint256 Y;
    }

    struct Fr {
        uint256 value;
    }

    // Encoding of field elements is: X[0] * z + X[1]
    struct G2Point {
        uint256[2] X;
        uint256[2] Y;
    }

    struct Proof {
        uint256[] public_input_values;
        G1Point[STATE_WIDTH] wire_commitments;
        G1Point grand_product_commitment;
        G1Point permutation_commitment;
        G1Point[STATE_WIDTH] quotient_poly_commitments;
        Fr[STATE_WIDTH] wire_values_at_z;
        Fr[STATE_WIDTH] wire_values_at_z_omega;
        Fr q_arith_at_z;
        Fr q_ecc_at_z;
        Fr q_c_at_z;
        Fr grand_product_at_z_omega;
        Fr quotient_polynomial_at_z;
        Fr linearization_polynomial_at_z;
        Fr[STATE_WIDTH - 1] permutation_polynomials_at_z;
        Fr wzBar;
        G1Point opening_at_z_proof;
        G1Point opening_at_z_omega_proof;
        G1Point[28] kate_group_elements;
        Fr[NUM_KATE_OPENING_ELEMENTS] kate_field_elements;
        uint256 kate_array_indexer;
        Fr debug_challenge;
        Fr[10] debug_markers;
    }

    struct PartialVerifierState {
        Fr alpha;
        Fr beta;
        Fr gamma;
        Fr[NUM_NU_CHALLENGES] v;
        Fr u;
        Fr zeta;
        Fr[] cached_lagrange_evals;
    }

    struct ChallengeTranscript {
        bytes32 debug_data;
        Fr init;
        Fr alpha;
        Fr beta;
        Fr gamma;
        Fr zeta;
        Fr[NUM_NU_CHALLENGES] v;
        Fr u;
        Fr alpha_base;
    }

    struct VerificationKey {
        uint256 domain_size;
        uint256 circuit_size;
        uint256 num_inputs;
        Fr domain_inverse;
        Fr work_root;
        Fr work_root_inverse;
        Fr omega;
        G1Point Q1;
        G1Point Q2;
        G1Point Q3;
        G1Point Q4;
        G1Point Q5;
        G1Point QM;
        G1Point QC;
        G1Point QARITH;
        G1Point QECC;
        G1Point QRANGE;
        G1Point QLOGIC;
        G1Point[STATE_WIDTH] sigma_commitments;
        Fr[STATE_WIDTH - 1] permutation_non_residues;
        G2Point g2_x;
    }

    struct BatchInversions {
        Fr public_input_delta_denominator_inverse;
        Fr zero_poly_inverse;
        Fr lagrange_1_fraction_inverse;
        Fr lagrange_n_fraction_inverse;
    }

    struct Fraction {
        Fr numerator;
        Fr denominator;
    }

    struct PartialStateFractions {
        Fraction public_input_delta;
        Fraction zero_poly;
        Fraction lagrange_1_fraction;
        Fraction lagrange_n_fraction;
    }
}




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





/**
 * @title PairingsBn254 library used for the fr, g1 and g2 point types
 * @dev Used to manipulate fr, g1, g2 types, perform modular arithmetic on them and call
 * the precompiles add, scalar mul and pairing
 *
 * Notes on optimisations
 * 1) Perform addmod, mulmod etc. in assembly - removes the check that Solidity performs to confirm that
 * the supplied modulus is not 0. This is safe as the modulus's used (r_mod, q_mod) are hard coded
 * inside the contract and not supplied by the user
 */
library PairingsBn254 {
    uint256 constant q_mod = 21888242871839275222246405745257275088696311157297823662689037894645226208583;
    uint256 constant r_mod = 21888242871839275222246405745257275088548364400416034343698204186575808495617;

    function new_fr(uint256 value) internal pure returns (Types.Fr memory out) {
        assembly {
            mstore(out, mod(value, r_mod))
        }
    }

    function copy(Types.Fr memory self)
        internal
        pure
        returns (Types.Fr memory n)
    {
        n.value = self.value;
    }

    function assign(Types.Fr memory self, Types.Fr memory other) internal pure {
        self.value = other.value;
    }

    function inverse(Types.Fr memory fr)
        internal
        view
        returns (Types.Fr memory)
    {
        assert(fr.value != 0);
        return pow(fr, r_mod - 2);
    }

    function add_assign(Types.Fr memory self, Types.Fr memory other)
        internal
        pure
    {
        assembly {
            mstore(self, addmod(mload(self), mload(other), r_mod))
        }
    }

    function add_fr(Types.Fr memory a, Types.Fr memory b)
        internal
        pure
        returns (Types.Fr memory out)
    {
        assembly {
            mstore(out, addmod(mload(a), mload(b), r_mod))
        }
    }

    // overloaded add_fr fn, to supply custom modulus
    function add_fr(
        Types.Fr memory a,
        Types.Fr memory b,
        uint256 modulus
    ) internal pure returns (Types.Fr memory out) {
        assembly {
            mstore(out, addmod(mload(a), mload(b), modulus))
        }
    }

    function sub_assign(Types.Fr memory self, Types.Fr memory other)
        internal
        pure
    {
        assembly {
            mstore(self, addmod(mload(self), sub(r_mod, mload(other)), r_mod))
        }
    }

    function sub_fr(Types.Fr memory a, Types.Fr memory b)
        internal
        pure
        returns (Types.Fr memory out)
    {
        assembly {
            mstore(out, addmod(mload(a), sub(r_mod, mload(b)), r_mod))
        }
    }

    function neg_assign(Types.Fr memory self) internal pure {
        assembly {
            mstore(self, mod(sub(r_mod, mload(self)), r_mod))
        }
    }

    function mul_assign(Types.Fr memory self, Types.Fr memory other)
        internal
        pure
    {
        assembly {
            mstore(self, mulmod(mload(self), mload(other), r_mod))
        }
    }

    function mul_fr(Types.Fr memory a, Types.Fr memory b)
        internal
        pure
        returns (Types.Fr memory out)
    {
        // uint256 mulValue;
        assembly {
            mstore(out, mulmod(mload(a), mload(b), r_mod))
        }
        // return Types.Fr(mulValue);
    }

    function sqr_fr(Types.Fr memory a)
        internal
        pure
        returns (Types.Fr memory out)
    {
        assembly {
            let aVal := mload(a)
            mstore(out, mulmod(aVal, aVal, r_mod))
        }
    }

    function pow_2(Types.Fr memory self) internal pure returns (Types.Fr memory) {
        uint256 input = self.value;

        assembly {
            input := mulmod(input, input, r_mod)
        }
        return Types.Fr(input);
    }

    function pow_3(Types.Fr memory self) internal pure returns (Types.Fr memory) {
        uint256 input = self.value;

        assembly {
            input := mulmod(input, input, r_mod)
            input := mulmod(input, mload(self), r_mod)
        }
        return Types.Fr(input);
    }

    function pow_4(Types.Fr memory self) internal pure returns (Types.Fr memory) {
        uint256 input = self.value;

        assembly {
            input := mulmod(input, input, r_mod)
            input := mulmod(input, input, r_mod)
        }
        return Types.Fr(input);
    }

    function get_msb(uint256 input) internal pure returns (uint256 bit_position) {
        assembly {
            input := or(input, shr(1, input))
            input := or(input, shr(2, input))
            input := or(input, shr(4, input))
            input := or(input, shr(8, input))
            input := or(input, shr(16, input))
            input := or(input, shr(32, input))
            input := or(input, shr(64, input))
            input := or(input, shr(128, input))
            input := shr(1, add(input, 1))
            let m := mload(0x40)
            mstore(m, 0x00)
            mstore(
                add(m, 0x1f),
                0x0000016d02d06e1303dad1e66f8e14310469dbdfd280e7b070948f3b15bb3200
            )
            mstore(
                add(m, 0x3f),
                0x05476ae3dc38e0fbd3c881fee89eb120712695d690c43caa1640bccb330c00ed
            )
            mstore(
                add(m, 0x5f),
                0x065248846b11e42fddae3900e1f9fc1ed4a8c9eb822dff1ce91a9fa1b26421a3
            )
            mstore(
                add(m, 0x7f),
                0x727a27b49600d7669144c5233d4faba517774174bd5ccc7c34c00d290058eeb6
            )
            mstore(
                add(m, 0x9f),
                0x075f539849f285006ccf12d9e58d3068de7faf933aba0046e237fac7fd9d1f25
            )
            mstore(
                add(m, 0xbf),
                0xd5c3a93fca0bec5183102ead00f81da7ea2c1b19a063a279b3006543224ea476
            )
            mstore(
                add(m, 0xdf),
                0x735b7bbf2857b55e97f100ced88c677e92b94536c69c24c23e0a500facf7a62b
            )
            mstore(
                add(m, 0xff),
                0x18627800424d755abe565df0cd8b7db8359bc1090ef62a61004c5955ef8ab79a
            )
            mstore(
                add(m, 0x11f),
                0x08f5604b548999f44a88f3878600000000000000000000000000000000000000
            )
            // let isolated_high_bit := and(input, sub(0, input))
            let index := mod(input, 269)
            bit_position := mload(add(m, index))
            bit_position := and(bit_position, 0xff)
        }
    }

    function pow_small(
        Types.Fr memory base,
        uint256 exp,
        uint256 mod
    ) internal pure returns (Types.Fr memory) {
        uint256 result = 1;
        uint256 input = base.value;
        for (uint256 count = 1; count <= exp; count *= 2) {
            if (exp & count != 0) {
                result = mulmod(result, input, mod);
            }
            input = mulmod(input, input, mod);
        }
        return new_fr(result);
    }

    function pow(Types.Fr memory self, uint256 power)
        internal
        view
        returns (Types.Fr memory)
    {
        uint256[6] memory input = [32, 32, 32, self.value, power, r_mod];
        uint256[1] memory result;
        bool success;
        assembly {
            success := staticcall(gas(), 0x05, input, 0xc0, result, 0x20)
        }
        require(success);
        return Types.Fr({value: result[0]});
    }

    // Calculates the result of an expression of the form: (a + bc + d).
    // a, b, c, d are Fr elements
    function compute_bracket(
        Types.Fr memory a,
        Types.Fr memory b,
        Types.Fr memory c,
        Types.Fr memory d
    ) internal pure returns (Types.Fr memory) {
        uint256 aPlusD;
        assembly {
            aPlusD := addmod(mload(a), mload(d), r_mod)
        }

        uint256 bMulC;
        assembly {
            bMulC := mulmod(mload(b), mload(c), r_mod)
        }

        uint256 result;
        assembly {
            result := addmod(aPlusD, bMulC, r_mod)
        }
        return new_fr(result);
    }

    // Calculates the result of an expression of the form: (abcd)
    // a, b, c are Fr elements
    // d is a G1Point
    function compute_product_3(
        Types.Fr memory a,
        Types.Fr memory b,
        Types.Fr memory c
    ) internal pure returns (Types.Fr memory) {
        Types.Fr memory scalar_product = mul_fr(a, mul_fr(b, c));
        return scalar_product;
    }

    // calculates the result of an expression of the form: (abc)
    // a, b are Fr elements
    // c is a G1Point
    function compute_product_3_mixed(
        Types.Fr memory a,
        Types.Fr memory b,
        Types.G1Point memory c
    ) internal view returns (Types.G1Point memory) {
        Types.Fr memory scalar_product = mul_fr(a, b);
        Types.G1Point memory result = point_mul(c, scalar_product);
        return result;
    }

    function compute_elliptic_mul(
        Types.G1Point memory first_term,
        Types.G1Point memory second_term,
        Types.G1Point memory third_term,
        Types.G1Point memory fourth_term,
        Types.G1Point memory fifth_term
    ) internal view returns (Types.G1Point memory) {
        Types.G1Point memory accumulator = copy_g1(first_term);
        accumulator = point_add(accumulator, second_term);
        accumulator = point_add(accumulator, third_term);
        accumulator = point_add(accumulator, fourth_term);
        accumulator = point_add(accumulator, fifth_term);
        return accumulator;
    }

    function accumulate_six(
        Types.G1Point memory first_term,
        Types.G1Point memory second_term,
        Types.G1Point memory third_term,
        Types.G1Point memory fourth_term,
        Types.G1Point memory fifth_term,
        Types.G1Point memory sixth_term
    ) internal view returns (Types.G1Point memory) {
        Types.G1Point memory accumulator = copy_g1(first_term);
        accumulator = point_add(accumulator, second_term);
        accumulator = point_add(accumulator, third_term);
        accumulator = point_add(accumulator, fourth_term);
        accumulator = point_add(accumulator, fifth_term);
        accumulator = point_add(accumulator, sixth_term);
        return accumulator;
    }

    function P1() internal pure returns (Types.G1Point memory) {
        return Types.G1Point(1, 2);
    }

    function new_g1(uint256 x, uint256 y)
        internal
        pure
        returns (Types.G1Point memory)
    {
        uint256 xValue;
        uint256 yValue;
        assembly {
            xValue := mod(x, r_mod)
            yValue := mod(y, r_mod)
        }
        return Types.G1Point(xValue, yValue);
    }

    function new_g2(uint256[2] memory x, uint256[2] memory y)
        internal
        pure
        returns (Types.G2Point memory)
    {
        return Types.G2Point(x, y);
    }

    function copy_g1(Types.G1Point memory self)
        internal
        pure
        returns (Types.G1Point memory result)
    {
        result.X = self.X;
        result.Y = self.Y;
    }

    function P2() internal pure returns (Types.G2Point memory) {
        // for some reason ethereum expects to have c1*v + c0 form

        return
            Types.G2Point(
                [
                    0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
                    0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed
                ],
                [
                    0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b,
                    0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa
                ]
            );
    }

    function negate(Types.G1Point memory self) internal pure {
        // The prime q in the base field F_q for G1
        if (self.X == 0 && self.Y == 0) return;
        self.Y = q_mod - self.Y;
    }

    function point_add(Types.G1Point memory p1, Types.G1Point memory p2)
        internal
        view
        returns (Types.G1Point memory r)
    {
        point_add_into_dest(p1, p2, r);
        return r;
    }

    function point_add_assign(Types.G1Point memory p1, Types.G1Point memory p2)
        internal
        view
    {
        point_add_into_dest(p1, p2, p1);
    }

    function point_add_into_dest(
        Types.G1Point memory p1,
        Types.G1Point memory p2,
        Types.G1Point memory dest
    ) internal view {
        validateG1Point(p1);
        validateG1Point(p2);
        uint256[4] memory input;
        if (p2.X == 0 && p2.Y == 0) {
            // we add zero, nothing happens
            dest.X = p1.X;
            dest.Y = p1.Y;
            return;
        } else if (p1.X == 0 && p1.Y == 0) {
            // we add into zero, and we add non-zero point
            dest.X = p2.X;
            dest.Y = p2.Y;
            return;
        } else {
            input[0] = p1.X;
            input[1] = p1.Y;
            input[2] = p2.X;
            input[3] = p2.Y;
        }
        bool success = false;
        assembly {
            success := staticcall(gas(), 6, input, 0x80, dest, 0x40)
        }
        require(success);
    }

    function point_sub_assign(Types.G1Point memory p1, Types.G1Point memory p2)
        internal
        view
    {
        point_sub_into_dest(p1, p2, p1);
    }

    function point_sub(Types.G1Point memory p1, Types.G1Point memory p2)
        internal
        view
        returns (Types.G1Point memory r)
    {
        point_sub_into_dest(p1, p2, r);
        return r;
    }

    function point_sub_into_dest(
        Types.G1Point memory p1,
        Types.G1Point memory p2,
        Types.G1Point memory dest
    ) internal view {
        validateG1Point(p1);
        validateG1Point(p2);
        uint256[4] memory input;
        if (p2.X == 0 && p2.Y == 0) {
            // we subtracted zero, nothing happens
            dest.X = p1.X;
            dest.Y = p1.Y;
            return;
        } else if (p1.X == 0 && p1.Y == 0) {
            // we subtract from zero, and we subtract non-zero point
            dest.X = p2.X;
            dest.Y = q_mod - p2.Y;
            return;
        } else {
            input[0] = p1.X;
            input[1] = p1.Y;
            input[2] = p2.X;
            input[3] = q_mod - p2.Y;
        }
        bool success = false;
        assembly {
            success := staticcall(gas(), 6, input, 0x80, dest, 0x40)
        }
        require(success);
    }

    function point_mul(Types.G1Point memory p, Types.Fr memory s)
        internal
        view
        returns (Types.G1Point memory r)
    {
        point_mul_into_dest(p, s, r);
        return r;
    }

    function point_mul_assign(Types.G1Point memory p, Types.Fr memory s)
        internal
        view
    {
        point_mul_into_dest(p, s, p);
    }

    function point_mul_into_dest(
        Types.G1Point memory p,
        Types.Fr memory s,
        Types.G1Point memory dest
    ) internal view {
        validateG1Point(p);
        validateScalar(s);
        uint256[3] memory input;
        input[0] = p.X;
        input[1] = p.Y;
        input[2] = s.value;
        bool success;

        assembly {
            success := staticcall(gas(), 7, input, 0x60, dest, 0x40)
        }
        require(success);
    }

    function pairing(Types.G1Point[] memory p1, Types.G2Point[] memory p2)
        internal
        view
        returns (bool)
    {
        require(p1.length == p2.length);

        for (uint256 i = 0; i < p1.length; i += 1) {
            validateG1Point(p1[i]);
        }
        uint256 elements = p1.length;
        uint256 inputSize = elements * 6;
        uint256[] memory input = new uint256[](inputSize);

        for (uint256 i = 0; i < elements; i++) {
            input[i * 6 + 0] = p1[i].X;
            input[i * 6 + 1] = p1[i].Y;
            input[i * 6 + 2] = p2[i].X[0];
            input[i * 6 + 3] = p2[i].X[1];
            input[i * 6 + 4] = p2[i].Y[0];
            input[i * 6 + 5] = p2[i].Y[1];
        }
        uint256[1] memory out;
        bool success;
        assembly {
            success := staticcall(
                gas(),
                8,
                add(input, 0x20),
                mul(inputSize, 0x20),
                out,
                0x20
            )
        }
        require(success);
        if (out[0] != 0) {
            return true;
        } else return false;
    }

    /// Convenience method for a pairing check for two pairs.
    function pairingProd2(
        Types.G1Point memory a1,
        Types.G2Point memory a2,
        Types.G1Point memory b1,
        Types.G2Point memory b2
    ) internal view returns (bool) {
        Types.G1Point[] memory p1 = new Types.G1Point[](2);
        Types.G2Point[] memory p2 = new Types.G2Point[](2);
        p1[0] = a1;
        p1[1] = b1;
        p2[0] = a2;
        p2[1] = b2;
        return pairing(p1, p2);
    }

    function validateG1Point(Types.G1Point memory point) internal pure {
        require(point.X < q_mod, "PairingsBn254: x > q_mod");
        require(point.Y < q_mod, "Pairng: y > q_mod");
        require(point.X != uint256(0), "PairingsBn254: x = 0");
        require(point.Y != uint256(0), "PairingsBn254: y = 0");

        // validating on curve: check y^2 = x^3 + 3 mod q_mod holds
        Types.Fr memory lhs = pow_small(new_fr(point.Y), 2, q_mod);
        Types.Fr memory rhs = add_fr(
            pow_small(new_fr(point.X), 3, q_mod),
            new_fr(3),
            q_mod
        );
        require(lhs.value == rhs.value, "PairingsBn254: not on curve");
    }

    function validateScalar(Types.Fr memory scalar) internal pure {
        require(scalar.value < r_mod, "PairingsBn254: scalar invalid");
    }
}






/**
 * @title Turbo Plonk polynomial evaluation
 * @dev Implementation of Turbo Plonk's polynomial evaluation algorithms
 *
 * Expected to be inherited by `TurboPlonk.sol`
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
library PolynomialEval {
    using PairingsBn254 for Types.G1Point;
    using PairingsBn254 for Types.G2Point;
    using PairingsBn254 for Types.Fr;

    /**
     * @dev Use batch inversion (so called Montgomery's trick). Circuit size is the domain
     * Allows multiple inversions to be performed in one inversion, at the expense of additional multiplications
     *
     * Returns a struct containing the inverted elements
     */
    function compute_batch_inversions(Types.PartialStateFractions memory partial_state_fractions)
        public
        view
        returns (
            Types.Fr memory,
            Types.Fr memory,
            Types.Fr[] memory
        )
    {
        uint256 denominatorsLength = 4;
        Types.Fr[] memory denominators = new Types.Fr[](denominatorsLength);

        // Extract all denominators from partial_state_fractions
        denominators[0] = partial_state_fractions.public_input_delta.denominator;
        denominators[1] = partial_state_fractions.zero_poly.denominator;
        denominators[2] = partial_state_fractions.lagrange_1_fraction.denominator;
        denominators[3] = partial_state_fractions.lagrange_n_fraction.denominator;

        Types.Fr memory product_accumulator = PairingsBn254.new_fr(1);

        Types.Fr[] memory temporaries = new Types.Fr[](denominatorsLength + 1);

        for (uint256 i = 0; i < denominatorsLength; i += 1) {
            temporaries[i] = product_accumulator;
            product_accumulator = PairingsBn254.mul_fr(product_accumulator, denominators[i]);
        }

        product_accumulator = product_accumulator.inverse();

        Types.Fr memory intermediate;
        for (uint256 i = denominatorsLength - 1; i < denominatorsLength; i -= 1) {
            intermediate = PairingsBn254.mul_fr(product_accumulator, temporaries[i]);
            product_accumulator = PairingsBn254.mul_fr(product_accumulator, denominators[i]);
            denominators[i] = intermediate;
        }

        Types.BatchInversions memory batch_inverted_elements = Types.BatchInversions({
            public_input_delta_denominator_inverse: denominators[0],
            zero_poly_inverse: denominators[1],
            lagrange_1_fraction_inverse: denominators[2],
            lagrange_n_fraction_inverse: denominators[3]
        });

        Types.Fr memory zero_polynomial_eval;
        Types.Fr memory public_input_delta;
        Types.Fr[] memory lagrange_evals;

        (zero_polynomial_eval, public_input_delta, lagrange_evals) = evaluate_fractions(
            partial_state_fractions,
            batch_inverted_elements
        );

        return (zero_polynomial_eval, public_input_delta, lagrange_evals);
    }

    function evaluate_fractions(
        Types.PartialStateFractions memory partial_state_fractions,
        Types.BatchInversions memory batch_inverted_elements
    )
        public
        pure
        returns (
            Types.Fr memory,
            Types.Fr memory,
            Types.Fr[] memory
        )
    {
        Types.Fr memory public_input_delta = PairingsBn254.mul_fr(
            batch_inverted_elements.public_input_delta_denominator_inverse,
            partial_state_fractions.public_input_delta.numerator
        );

        Types.Fr memory zero_poly_eval = PairingsBn254.mul_fr(
            batch_inverted_elements.zero_poly_inverse,
            partial_state_fractions.zero_poly.numerator
        );

        Types.Fr memory L1 = PairingsBn254.mul_fr(
            batch_inverted_elements.lagrange_1_fraction_inverse,
            partial_state_fractions.lagrange_1_fraction.numerator
        );

        Types.Fr memory Ln = PairingsBn254.mul_fr(
            batch_inverted_elements.lagrange_n_fraction_inverse,
            partial_state_fractions.lagrange_n_fraction.numerator
        );

        Types.Fr[] memory lagrange_evals = new Types.Fr[](2);
        lagrange_evals[0] = L1;
        lagrange_evals[1] = Ln;

        return (zero_poly_eval, public_input_delta, lagrange_evals);
    }

    function compute_zero_polynomial(
        Types.Fr memory zeta,
        uint256 circuit_size,
        Types.Fr memory work_root_inverse
    ) public pure returns (Types.Fraction memory) {
        Types.Fr memory firstTerm = zeta.pow_small(circuit_size, Types.r_mod);

        Types.Fr memory secondTerm = PairingsBn254.new_fr(1);

        Types.Fr memory thirdTerm = zeta;
        Types.Fr memory fourthTerm = work_root_inverse;

        Types.Fr memory numerator = PairingsBn254.sub_fr(firstTerm, secondTerm);
        Types.Fr memory denominator = PairingsBn254.sub_fr(thirdTerm, fourthTerm);

        return Types.Fraction({numerator: numerator, denominator: denominator});
    }

    function compute_public_input_delta(
        uint256[] memory public_inputs,
        Types.ChallengeTranscript memory challenges,
        Types.VerificationKey memory vk
    ) internal pure returns (Types.Fraction memory) {
        Types.Fr memory numerator = PairingsBn254.new_fr(1);
        Types.Fr memory denominator = PairingsBn254.new_fr(1);

        Types.Fr memory T0 = PairingsBn254.new_fr(0);
        Types.Fr memory T1 = PairingsBn254.new_fr(0);
        Types.Fr memory T2 = PairingsBn254.new_fr(0);
        Types.Fr memory T3 = PairingsBn254.new_fr(0);

        Types.Fr memory accumulating_root = PairingsBn254.new_fr(1);

        for (uint256 index = 0; index < public_inputs.length; index += 1) {
            T0 = PairingsBn254.new_fr(public_inputs[index]).add_fr(challenges.gamma);
            T1 = accumulating_root.mul_fr(challenges.beta);
            T2 = T1.mul_fr(PairingsBn254.new_fr(Types.coset_generator0));
            T3 = T1.mul_fr(PairingsBn254.new_fr(Types.coset_generator7));
            T2.add_assign(T0);
            T3.add_assign(T0);
            numerator.mul_assign(T2);
            denominator.mul_assign(T3);
            accumulating_root.mul_assign(vk.work_root);
        }

        return Types.Fraction({numerator: numerator, denominator: denominator});
    }

    /**
     * @dev Computes the lagrange evaluations L1 and Ln.
     * @return Returns lagrange evals as an array, with L1 at index 0 and Ln at index 1
     */
    function compute_lagrange_evaluations(Types.VerificationKey memory vk, Types.Fr memory zeta)
        public
        pure
        returns (Types.Fraction[] memory)
    {
        Types.Fr memory zeta_copy = zeta;
        Types.Fr memory vanishing_poly_numerator = PairingsBn254.sub_fr(
            zeta.pow_small(vk.circuit_size, Types.r_mod),
            PairingsBn254.new_fr(1)
        );

        Types.Fr memory domain_inverse = vk.domain_inverse;

        Types.Fr memory numerator = PairingsBn254.mul_fr(vanishing_poly_numerator, domain_inverse);

        Types.Fr memory denominator1 = PairingsBn254.sub_fr(zeta_copy, PairingsBn254.new_fr(1));

        Types.Fr memory T0 = PairingsBn254.mul_fr(zeta_copy, vk.work_root.pow_2());
        Types.Fr memory denominatorN = PairingsBn254.sub_fr(T0, PairingsBn254.new_fr(1));

        Types.Fraction memory L1 = Types.Fraction({numerator: numerator, denominator: denominator1});
        Types.Fraction memory Ln = Types.Fraction({numerator: numerator, denominator: denominatorN});

        Types.Fraction[] memory lagrange_evals = new Types.Fraction[](2);
        lagrange_evals[0] = L1;
        lagrange_evals[1] = Ln;

        return lagrange_evals;
    }

    function compute_arithmetic_gate_quotient_contribution(
        Types.ChallengeTranscript memory challenges,
        Types.Proof memory proof
    ) public pure returns (Types.Fr memory) {
        Types.Fr memory t1 = proof.q_arith_at_z.mul_fr(proof.q_arith_at_z);
        t1.sub_assign(proof.q_arith_at_z);

        Types.Fr memory t2 = proof.wire_values_at_z[3].add_fr(proof.wire_values_at_z[3]);
        t2.add_assign(t2);
        t2.neg_assign();
        t2.add_assign(proof.wire_values_at_z[2]);

        Types.Fr memory t3 = t2.mul_fr(t2);
        t3.add_assign(t3);

        Types.Fr memory t4 = t2.add_fr(t2);
        t4.add_assign(t2);

        Types.Fr memory t5 = t4.add_fr(t4);
        t4.add_assign(t5);

        t4.sub_assign(t3);
        t4.sub_assign(Types.Fr(7));

        t2.mul_assign(t4);

        t1.mul_assign(t2);

        t1.mul_assign(challenges.alpha_base);

        // update alpha
        challenges.alpha_base.mul_assign(challenges.alpha);
        challenges.alpha_base.mul_assign(challenges.alpha);

        return t1;
    }

    function compute_pedersen_gate_quotient_contribution(
        Types.ChallengeTranscript memory challenges,
        Types.Proof memory proof
    ) public pure returns (Types.Fr memory) {
        Types.Fr memory delta = proof.wire_values_at_z_omega[3].sub_fr(proof.wire_values_at_z[3]);
        delta.sub_assign(proof.wire_values_at_z[3]);
        delta.sub_assign(proof.wire_values_at_z[3]);
        delta.sub_assign(proof.wire_values_at_z[3]);

        Types.Fr memory t0 = Types.Fr(0);
        Types.Fr memory t1 = Types.Fr(0);
        Types.Fr memory t2 = Types.Fr(0);
        Types.Fr memory gate_identity = Types.Fr(0);

        {
            Types.Fr memory accumulator_identity = delta.add_fr(Types.Fr(1));
            accumulator_identity.mul_assign(PairingsBn254.add_fr(delta, Types.Fr(3)));
            accumulator_identity.mul_assign(PairingsBn254.sub_fr(delta, Types.Fr(1)));
            accumulator_identity.mul_assign(PairingsBn254.sub_fr(delta, Types.Fr(3)));
            accumulator_identity.mul_assign(challenges.alpha_base);

            gate_identity.add_assign(accumulator_identity);
            // update alpha base
            challenges.alpha_base.mul_assign(challenges.alpha);
        }

        {
            Types.Fr memory x_alpha_identity = proof.wire_values_at_z_omega[2].mul_fr(challenges.alpha_base);
            x_alpha_identity.neg_assign();

            gate_identity.add_assign(x_alpha_identity);

            // update alpha base
            challenges.alpha_base.mul_assign(challenges.alpha);
        }

        {
            t0 = proof.wire_values_at_z_omega[0].add_fr(proof.wire_values_at_z[0]);
            t0.add_assign(proof.wire_values_at_z_omega[2]);

            t1 = proof.wire_values_at_z_omega[2].sub_fr(proof.wire_values_at_z[0]);
            t1.mul_assign(t1); // TODO CHECK

            t0.mul_assign(t1);

            t1 = proof.wire_values_at_z_omega[2].mul_fr(proof.wire_values_at_z_omega[2]);
            t1.mul_assign(proof.wire_values_at_z_omega[2]);

            t2 = proof.wire_values_at_z[1].mul_fr(proof.wire_values_at_z[1]);

            t1.add_assign(t2);
            t1.sub_assign(PairingsBn254.new_fr(17)); // grumkin curve b parameter (y^2 = x^3 - 17)
            t1.neg_assign();

            t2 = delta.mul_fr(proof.wire_values_at_z[1]);
            t2.mul_assign(proof.q_ecc_at_z);
            t2.add_assign(t2);

            Types.Fr memory x_accumulator_identity = t0.add_fr(t1);
            x_accumulator_identity.add_assign(t2);
            x_accumulator_identity.mul_assign(challenges.alpha_base);

            gate_identity.add_assign(x_accumulator_identity);

            // update alpha base
            challenges.alpha_base.mul_assign(challenges.alpha);
        }

        {
            t0 = proof.wire_values_at_z_omega[1].add_fr(proof.wire_values_at_z[1]);

            t1 = proof.wire_values_at_z_omega[2].sub_fr(proof.wire_values_at_z[0]);

            t0.mul_assign(t1);

            t1 = proof.wire_values_at_z[0].sub_fr(proof.wire_values_at_z_omega[0]);

            t2 = proof.q_ecc_at_z.mul_fr(delta);
            t2.neg_assign();
            t2.add_assign(proof.wire_values_at_z[1]);

            t1.mul_assign(t2);

            Types.Fr memory y_accumulator_identity = t0.add_fr(t1);
            y_accumulator_identity.mul_assign(challenges.alpha_base);

            gate_identity.add_assign(y_accumulator_identity);

            // update alpha base
            challenges.alpha_base.mul_assign(challenges.alpha);
        }

        {
            Types.Fr memory accumulator_init_identity = proof.wire_values_at_z[3].sub_fr(Types.Fr(1));

            t1 = accumulator_init_identity.sub_fr(proof.wire_values_at_z[2]);

            accumulator_init_identity.mul_assign(t1);
            accumulator_init_identity.mul_assign(challenges.alpha_base);
            accumulator_init_identity.mul_assign(proof.q_c_at_z);

            gate_identity.add_assign(accumulator_init_identity);

            // update alpha base
            challenges.alpha_base.mul_assign(challenges.alpha);
        }

        {
            Types.Fr memory x_init_identity = proof.wire_values_at_z[0].mul_fr(proof.wire_values_at_z[2]);
            x_init_identity.mul_assign(challenges.alpha_base);
            x_init_identity.neg_assign();
            x_init_identity.mul_assign(proof.q_c_at_z);

            gate_identity.add_assign(x_init_identity);

            // update alpha base
            challenges.alpha_base.mul_assign(challenges.alpha);
        }

        {
            Types.Fr memory y_init_identity = Types.Fr(1).sub_fr(proof.wire_values_at_z[3]);
            y_init_identity.mul_assign(proof.q_c_at_z);

            t1 = proof.wire_values_at_z[1].mul_fr(proof.wire_values_at_z[2]);

            y_init_identity.sub_assign(t1);
            y_init_identity.mul_assign(challenges.alpha_base);
            y_init_identity.mul_assign(proof.q_c_at_z);

            gate_identity.add_assign(y_init_identity);

            // update alpha base
            challenges.alpha_base.mul_assign(challenges.alpha);
        }

        gate_identity.mul_assign(proof.q_ecc_at_z);

        return gate_identity;
    }

    function compute_permutation_quotient_contribution(
        Types.Fr memory public_input_delta,
        Types.ChallengeTranscript memory challenges,
        Types.Fr[] memory lagrange_evals,
        Types.Proof memory proof
    ) public pure returns (Types.Fr memory) {
        Types.Fr memory numerator_collector = Types.Fr(0);

        // first term
        numerator_collector.add_assign(proof.linearization_polynomial_at_z);

        // second term
        Types.Fr memory first_bracket = PairingsBn254.compute_bracket(
            proof.wire_values_at_z[0],
            challenges.beta,
            proof.permutation_polynomials_at_z[0],
            challenges.gamma
        );
        Types.Fr memory second_bracket = PairingsBn254.compute_bracket(
            proof.wire_values_at_z[1],
            challenges.beta,
            proof.permutation_polynomials_at_z[1],
            challenges.gamma
        );
        Types.Fr memory third_bracket = PairingsBn254.compute_bracket(
            proof.wire_values_at_z[2],
            challenges.beta,
            proof.permutation_polynomials_at_z[2],
            challenges.gamma
        );
        first_bracket.mul_assign(second_bracket);
        first_bracket.mul_assign(third_bracket);
        first_bracket.mul_assign(PairingsBn254.add_fr(proof.wire_values_at_z[3], challenges.gamma));
        first_bracket.mul_assign(proof.grand_product_at_z_omega);
        first_bracket.mul_assign(challenges.alpha);
        numerator_collector.sub_assign(first_bracket);

        // third term
        Types.Fr memory third_term = PairingsBn254.mul_fr(lagrange_evals[0], challenges.alpha.pow_3());
        numerator_collector.sub_assign(third_term);

        // fourth term
        Types.Fr memory fourth_term = PairingsBn254.mul_fr(lagrange_evals[1], challenges.alpha.pow_2());
        Types.Fr memory temp = PairingsBn254.sub_fr(proof.grand_product_at_z_omega, public_input_delta);
        fourth_term.mul_assign(temp);
        numerator_collector.add_assign(fourth_term);

        challenges.alpha_base.mul_assign(challenges.alpha);
        challenges.alpha_base.mul_assign(challenges.alpha);
        challenges.alpha_base.mul_assign(challenges.alpha);

        return numerator_collector;
    }

    function compute_quotient_polynomial(
        Types.Fr memory zero_poly_eval,
        Types.Fr memory public_input_delta,
        Types.ChallengeTranscript memory challenges,
        Types.Fr[] memory lagrange_evals,
        Types.Proof memory proof
    ) public view returns (Types.Fr memory) {
        Types.Fr memory t0 = compute_permutation_quotient_contribution(
            public_input_delta,
            challenges,
            lagrange_evals,
            proof
        );

        Types.Fr memory t1 = compute_arithmetic_gate_quotient_contribution(challenges, proof);

        Types.Fr memory t2 = compute_pedersen_gate_quotient_contribution(challenges, proof);

        Types.Fr memory quotient_eval = t0.add_fr(t1);
        quotient_eval.add_assign(t2);
        quotient_eval.mul_assign(zero_poly_eval.inverse());

        return quotient_eval;
    }

    function compute_partial_opening_commitment(
        Types.ChallengeTranscript memory challenges,
        Types.Fr memory L1_fr,
        Types.G1Point memory,
        Types.VerificationKey memory vk,
        Types.Proof memory proof
    ) public view returns (Types.G1Point memory) {
        (Types.G1Point memory accumulator, ) = compute_grand_product_opening_scalar(proof, vk, challenges, L1_fr);
        (Types.G1Point memory arithmetic_term, ) = compute_arithmetic_gate_opening_scalars(proof, vk, challenges);
        (Types.G1Point memory range_term, ) = compute_range_gate_opening_scalar(proof, vk, challenges);
        (Types.G1Point memory logic_term, ) = compute_logic_gate_opening_scalar(proof, vk, challenges);

        accumulator.point_add_assign(arithmetic_term);
        accumulator.point_add_assign(range_term);
        accumulator.point_add_assign(logic_term);
        return accumulator;
    }

    function compute_batch_opening_commitment(
        Types.ChallengeTranscript memory challenges,
        Types.VerificationKey memory vk,
        Types.G1Point memory partial_opening_commitment,
        Types.Proof memory proof
    ) public view returns (Types.G1Point memory) {
        // first term

        Types.G1Point memory accumulator = PairingsBn254.copy_g1(proof.quotient_poly_commitments[0]); //tlow

        // second term
        Types.Fr memory zeta_n = challenges.zeta.pow_small(vk.circuit_size, Types.r_mod);

        accumulator.point_add_assign(PairingsBn254.point_mul(proof.quotient_poly_commitments[1], zeta_n));

        // third term
        Types.Fr memory zeta_2n = zeta_n.pow_2();

        accumulator.point_add_assign(PairingsBn254.point_mul(proof.quotient_poly_commitments[2], zeta_2n));

        // fourth term
        Types.Fr memory zeta_3n = zeta_n.pow_3();

        accumulator.point_add_assign(PairingsBn254.point_mul(proof.quotient_poly_commitments[3], zeta_3n));

        // fifth term
        accumulator.point_add_assign(partial_opening_commitment);

        Types.Fr memory u_plus_one = challenges.u.add_fr(Types.Fr(1));

        // shifted_wire_value
        Types.Fr memory scalar_multiplier = challenges.v[0].mul_fr(u_plus_one);

        accumulator.point_add_assign(PairingsBn254.point_mul(proof.wire_commitments[0], scalar_multiplier));

        scalar_multiplier = challenges.v[1].mul_fr(u_plus_one);
        accumulator.point_add_assign(PairingsBn254.point_mul(proof.wire_commitments[1], scalar_multiplier));

        scalar_multiplier = challenges.v[2].mul_fr(u_plus_one);
        accumulator.point_add_assign(PairingsBn254.point_mul(proof.wire_commitments[2], scalar_multiplier));

        scalar_multiplier = challenges.v[3].mul_fr(u_plus_one);
        accumulator.point_add_assign(PairingsBn254.point_mul(proof.wire_commitments[3], scalar_multiplier));

        // copy permutation selectors
        scalar_multiplier = challenges.v[4];
        accumulator.point_add_assign(PairingsBn254.point_mul(vk.sigma_commitments[0], scalar_multiplier));

        scalar_multiplier = challenges.v[5];
        accumulator.point_add_assign(PairingsBn254.point_mul(vk.sigma_commitments[1], scalar_multiplier));

        scalar_multiplier = challenges.v[6];
        accumulator.point_add_assign(PairingsBn254.point_mul(vk.sigma_commitments[2], scalar_multiplier));

        // arithmetic selector evaluations
        scalar_multiplier = challenges.v[7];
        accumulator.point_add_assign(PairingsBn254.point_mul(vk.QARITH, scalar_multiplier));

        // arithmetic selector evaluations
        scalar_multiplier = challenges.v[8];
        accumulator.point_add_assign(PairingsBn254.point_mul(vk.QECC, scalar_multiplier));

        return accumulator;
    }

    function compute_batch_evaluation_commitment(Types.Proof memory proof, Types.ChallengeTranscript memory challenges)
        public
        view
        returns (Types.G1Point memory, Types.Fr memory)
    {
        Types.Fr memory kate_opening_scalar = Types.Fr(0);

        kate_opening_scalar.add_assign(challenges.v[0].mul_fr(proof.wire_values_at_z[0]));
        kate_opening_scalar.add_assign(challenges.v[1].mul_fr(proof.wire_values_at_z[1]));
        kate_opening_scalar.add_assign(challenges.v[2].mul_fr(proof.wire_values_at_z[2]));
        kate_opening_scalar.add_assign(challenges.v[3].mul_fr(proof.wire_values_at_z[3]));
        kate_opening_scalar.add_assign(challenges.v[4].mul_fr(proof.permutation_polynomials_at_z[0]));
        kate_opening_scalar.add_assign(challenges.v[5].mul_fr(proof.permutation_polynomials_at_z[1]));
        kate_opening_scalar.add_assign(challenges.v[6].mul_fr(proof.permutation_polynomials_at_z[2]));
        kate_opening_scalar.add_assign(challenges.v[7].mul_fr(proof.q_arith_at_z));
        kate_opening_scalar.add_assign(challenges.v[8].mul_fr(proof.q_ecc_at_z));
        kate_opening_scalar.add_assign(challenges.v[9].mul_fr(proof.q_c_at_z));
        kate_opening_scalar.add_assign(challenges.v[0].mul_fr(challenges.u).mul_fr(proof.wire_values_at_z_omega[0]));
        kate_opening_scalar.add_assign(challenges.v[1].mul_fr(challenges.u).mul_fr(proof.wire_values_at_z_omega[1]));
        kate_opening_scalar.add_assign(challenges.v[2].mul_fr(challenges.u).mul_fr(proof.wire_values_at_z_omega[2]));
        kate_opening_scalar.add_assign(challenges.v[3].mul_fr(challenges.u).mul_fr(proof.wire_values_at_z_omega[3]));
        kate_opening_scalar.add_assign(challenges.v[10].mul_fr(proof.linearization_polynomial_at_z));
        kate_opening_scalar.add_assign(challenges.u.mul_fr(proof.grand_product_at_z_omega));
        kate_opening_scalar.add_assign(proof.quotient_polynomial_at_z);

        Types.G1Point memory batch_eval_commitment = PairingsBn254.point_mul(PairingsBn254.P1(), kate_opening_scalar);
        return (batch_eval_commitment, kate_opening_scalar);
    }

    // Compute kate opening scalar for arithmetic gate selectors
    function compute_arithmetic_gate_opening_scalars(
        Types.Proof memory proof,
        Types.VerificationKey memory vk,
        Types.ChallengeTranscript memory challenges
    ) public view returns (Types.G1Point memory, Types.Fr[7] memory scalar_multipliers) {
        // multiplication gate selector
        scalar_multipliers[0] = proof.wire_values_at_z[0].mul_fr(proof.wire_values_at_z[1]);
        scalar_multipliers[0].mul_assign(challenges.v[10]);
        scalar_multipliers[0].mul_assign(challenges.alpha_base);
        scalar_multipliers[0].mul_assign(proof.q_arith_at_z);

        // 1st wire selector
        scalar_multipliers[1] = proof.wire_values_at_z[0].mul_fr(challenges.v[10]);
        scalar_multipliers[1].mul_assign(challenges.alpha_base);
        scalar_multipliers[1].mul_assign(proof.q_arith_at_z);

        // 2nd wire selector
        scalar_multipliers[2] = proof.wire_values_at_z[1].mul_fr(challenges.v[10]);
        scalar_multipliers[2].mul_assign(challenges.alpha_base);
        scalar_multipliers[2].mul_assign(proof.q_arith_at_z);

        // 3rd wire selector
        scalar_multipliers[3] = proof.wire_values_at_z[2].mul_fr(challenges.v[10]);
        scalar_multipliers[3].mul_assign(challenges.alpha_base);
        scalar_multipliers[3].mul_assign(proof.q_arith_at_z);

        // 4th wire selector
        scalar_multipliers[4] = proof.wire_values_at_z[3].mul_fr(challenges.v[10]);
        scalar_multipliers[4].mul_assign(challenges.alpha_base);
        scalar_multipliers[4].mul_assign(proof.q_arith_at_z);

        // 5th wire selector
        scalar_multipliers[5] = proof.wire_values_at_z[3].mul_fr(proof.wire_values_at_z[3]);
        scalar_multipliers[5].sub_assign(proof.wire_values_at_z[3]);
        scalar_multipliers[5].mul_assign(PairingsBn254.sub_fr(proof.wire_values_at_z[3], Types.Fr(2)));
        scalar_multipliers[5].mul_assign(challenges.alpha_base);
        scalar_multipliers[5].mul_assign(challenges.alpha);
        scalar_multipliers[5].mul_assign(proof.q_arith_at_z);
        scalar_multipliers[5].mul_assign(challenges.v[10]);

        // constant wire selector
        scalar_multipliers[6] = challenges.v[10].mul_fr(challenges.alpha_base);
        scalar_multipliers[6].mul_assign(proof.q_arith_at_z);

        // TurboPlonk requires an explicit evaluation of q_c
        scalar_multipliers[6].add_assign(challenges.v[9]);

        // update alpha_base
        challenges.alpha_base.mul_assign(challenges.alpha);
        challenges.alpha_base.mul_assign(challenges.alpha);

        compute_pedersen_gate_opening_scalars(proof, challenges, scalar_multipliers);

        Types.G1Point memory accumulator = PairingsBn254.point_mul(vk.QM, scalar_multipliers[0]);

        Types.G1Point memory to_add = PairingsBn254.point_mul(vk.Q1, scalar_multipliers[1]);
        accumulator.point_add_assign(to_add);

        to_add = PairingsBn254.point_mul(vk.Q2, scalar_multipliers[2]);
        accumulator.point_add_assign(to_add);

        to_add = PairingsBn254.point_mul(vk.Q3, scalar_multipliers[3]);
        accumulator.point_add_assign(to_add);

        to_add = PairingsBn254.point_mul(vk.Q4, scalar_multipliers[4]);
        accumulator.point_add_assign(to_add);

        to_add = PairingsBn254.point_mul(vk.Q5, scalar_multipliers[5]);
        accumulator.point_add_assign(to_add);

        to_add = PairingsBn254.point_mul(vk.QC, scalar_multipliers[6]);
        accumulator.point_add_assign(to_add);

        return (accumulator, scalar_multipliers);
    }

    // Compute kate opening scalar for arithmetic gate selectors
    function compute_pedersen_gate_opening_scalars(
        Types.Proof memory proof,
        Types.ChallengeTranscript memory challenges,
        Types.Fr[7] memory scalar_multipliers
    ) public pure {
        Types.Fr memory delta = proof.wire_values_at_z_omega[3].sub_fr(proof.wire_values_at_z[3]);
        delta.sub_assign(proof.wire_values_at_z[3]);
        delta.sub_assign(proof.wire_values_at_z[3]);
        delta.sub_assign(proof.wire_values_at_z[3]);

        Types.Fr[7] memory alpha_powers;
        alpha_powers[0] = PairingsBn254.new_fr(challenges.alpha_base.value);
        alpha_powers[1] = alpha_powers[0].mul_fr(challenges.alpha);
        alpha_powers[2] = alpha_powers[1].mul_fr(challenges.alpha);
        alpha_powers[3] = alpha_powers[2].mul_fr(challenges.alpha);
        alpha_powers[4] = alpha_powers[3].mul_fr(challenges.alpha);
        alpha_powers[5] = alpha_powers[4].mul_fr(challenges.alpha);
        alpha_powers[6] = alpha_powers[5].mul_fr(challenges.alpha);

        Types.Fr[6] memory multiplicands;
        multiplicands[1] = delta.mul_fr(delta);
        multiplicands[1].mul_assign(proof.q_ecc_at_z);
        multiplicands[1].mul_assign(alpha_powers[1]); // TODO CHECK

        multiplicands[2] = alpha_powers[1].mul_fr(proof.q_ecc_at_z);

        multiplicands[3] = proof.wire_values_at_z_omega[0].copy();
        multiplicands[3].sub_assign(proof.wire_values_at_z[0]);
        multiplicands[3].mul_assign(delta);
        multiplicands[3].mul_assign(proof.wire_values_at_z_omega[2]);
        multiplicands[3].mul_assign(alpha_powers[3]);
        // multiplicands[3].mul_assign(proof.q_ecc_at_z);

        Types.Fr memory t1 = delta.mul_fr(proof.wire_values_at_z_omega[2]);
        t1.mul_assign(proof.wire_values_at_z[1]);
        t1.mul_assign(alpha_powers[2]);
        t1.add_assign(t1);
        // t1.mul_assign(proof.q_ecc_at_z);

        multiplicands[3].add_assign(t1);
        multiplicands[3].mul_assign(proof.q_ecc_at_z);

        multiplicands[4] = proof.wire_values_at_z[2].mul_fr(proof.q_ecc_at_z);
        multiplicands[4].mul_assign(proof.q_c_at_z);
        multiplicands[4].mul_assign(alpha_powers[5]);

        multiplicands[5] = Types.Fr(1).sub_fr(proof.wire_values_at_z[3]);
        multiplicands[5].mul_assign(proof.q_ecc_at_z);
        multiplicands[5].mul_assign(proof.q_c_at_z);
        multiplicands[5].mul_assign(alpha_powers[5]);

        multiplicands[0] = proof.wire_values_at_z[2].mul_fr(proof.q_ecc_at_z);
        multiplicands[0].mul_assign(proof.q_c_at_z);
        multiplicands[0].mul_assign(alpha_powers[6]);

        scalar_multipliers[0].add_assign(multiplicands[0].mul_fr(challenges.v[10]));
        scalar_multipliers[1].add_assign(multiplicands[1].mul_fr(challenges.v[10]));
        scalar_multipliers[2].add_assign(multiplicands[2].mul_fr(challenges.v[10]));
        scalar_multipliers[3].add_assign(multiplicands[3].mul_fr(challenges.v[10]));
        scalar_multipliers[4].add_assign(multiplicands[4].mul_fr(challenges.v[10]));
        scalar_multipliers[5].add_assign(multiplicands[5].mul_fr(challenges.v[10]));

        challenges.alpha_base = alpha_powers[6].mul_fr(challenges.alpha);
    }

    // Compute kate opening scalar for arithmetic gate selectors
    function compute_logic_gate_opening_scalar(
        Types.Proof memory proof,
        Types.VerificationKey memory vk,
        Types.ChallengeTranscript memory challenges
    ) public view returns (Types.G1Point memory, Types.Fr memory) {
        proof.debug_challenge = PairingsBn254.new_fr(challenges.alpha_base.value);
        Types.Fr memory t0 = proof.wire_values_at_z[0].add_fr(proof.wire_values_at_z[0]);
        t0.add_assign(t0);
        t0.neg_assign();
        t0.add_assign(proof.wire_values_at_z_omega[0]);

        Types.Fr memory t1 = proof.wire_values_at_z[1].add_fr(proof.wire_values_at_z[1]);
        t1.add_assign(t1);
        t1.neg_assign();
        t1.add_assign(proof.wire_values_at_z_omega[1]);

        Types.Fr memory delta_sum = t0.add_fr(t1);

        Types.Fr memory t2 = t0.mul_fr(t0);
        Types.Fr memory t3 = t1.mul_fr(t1);

        Types.Fr memory delta_squared_sum = t2.add_fr(t3);

        Types.Fr memory identity = delta_sum.mul_fr(delta_sum);
        identity.sub_assign(delta_squared_sum);

        Types.Fr memory t4 = proof.wire_values_at_z[2].add_fr(proof.wire_values_at_z[2]);
        identity.sub_assign(t4);
        identity.mul_assign(challenges.alpha);

        t4.add_assign(t4);
        t2.sub_assign(t0);
        t0.add_assign(t0);
        t0.add_assign(t0);
        t0 = t2.sub_fr(t0);
        t0.add_assign(Types.Fr(6));

        t0.mul_assign(t2);
        identity.add_assign(t0);
        identity.mul_assign(challenges.alpha);

        t3.sub_assign(t1);
        t1.add_assign(t1);
        t1.add_assign(t1);
        t1 = t3.sub_fr(t1);
        t1.add_assign(Types.Fr(6));

        t1.mul_assign(t3);
        identity.add_assign(t1);
        identity.mul_assign(challenges.alpha);

        t0 = delta_sum.add_fr(delta_sum);
        t0.add_assign(delta_sum);

        t1 = t0.add_fr(t0);
        t1.add_assign(t0);

        delta_sum = t1.add_fr(t1);

        t2 = delta_sum.add_fr(delta_sum);
        t2.add_assign(t2);
        t1.add_assign(t2);

        t2 = delta_squared_sum.add_fr(delta_squared_sum);
        t2.add_assign(delta_squared_sum);

        delta_squared_sum = t2.add_fr(t2);
        delta_squared_sum.add_assign(t2);
        delta_squared_sum.add_assign(delta_squared_sum);

        delta_sum = t4.sub_fr(delta_sum);
        delta_sum.add_assign(Types.Fr(81));
        delta_sum.mul_assign(proof.wire_values_at_z[2]);

        t1 = delta_squared_sum.sub_fr(t1);
        t1.add_assign(Types.Fr(83));

        delta_sum.add_assign(t1);
        delta_sum.mul_assign(proof.wire_values_at_z[2]);

        t2 = proof.wire_values_at_z[3].add_fr(proof.wire_values_at_z[3]);
        t2.add_assign(t2);
        t2 = proof.wire_values_at_z_omega[3].sub_fr(t2);
        t3 = t2.add_fr(t2);
        t2.add_assign(t3);

        t3 = t2.add_fr(t2);
        t3.add_assign(t2);

        t3.sub_assign(t0);
        t3.mul_assign(proof.q_c_at_z);

        t2.add_assign(t0);
        delta_sum.add_assign(delta_sum);
        t2.sub_assign(delta_sum);

        t2.add_assign(t3);

        identity.add_assign(t2);
        identity.mul_assign(challenges.alpha_base);
        identity.mul_assign(challenges.v[10]);

        Types.G1Point memory kate_component = PairingsBn254.point_mul(vk.QLOGIC, identity);

        challenges.alpha_base.mul_assign(challenges.alpha);
        challenges.alpha_base.mul_assign(challenges.alpha);
        challenges.alpha_base.mul_assign(challenges.alpha);
        challenges.alpha_base.mul_assign(challenges.alpha);

        return (kate_component, identity);
    }

    // Compute kate opening scalar for arithmetic gate selectors
    function compute_range_gate_opening_scalar(
        Types.Proof memory proof,
        Types.VerificationKey memory vk,
        Types.ChallengeTranscript memory challenges
    ) public view returns (Types.G1Point memory, Types.Fr memory) {
        Types.Fr[4] memory alpha_powers;
        alpha_powers[0] = PairingsBn254.new_fr(challenges.alpha_base.value);
        alpha_powers[1] = alpha_powers[0].mul_fr(challenges.alpha);
        alpha_powers[2] = alpha_powers[1].mul_fr(challenges.alpha);
        alpha_powers[3] = alpha_powers[2].mul_fr(challenges.alpha);

        Types.Fr memory delta_1 = proof.wire_values_at_z[3].add_fr(proof.wire_values_at_z[3]);
        delta_1.add_assign(delta_1);
        delta_1.neg_assign();
        delta_1.add_assign(proof.wire_values_at_z[2]);

        Types.Fr memory delta_2 = proof.wire_values_at_z[2].add_fr(proof.wire_values_at_z[2]);
        delta_2.add_assign(delta_2);
        delta_2.neg_assign();
        delta_2.add_assign(proof.wire_values_at_z[1]);

        Types.Fr memory delta_3 = proof.wire_values_at_z[1].add_fr(proof.wire_values_at_z[1]);
        delta_3.add_assign(delta_3);
        delta_3.neg_assign();
        delta_3.add_assign(proof.wire_values_at_z[0]);

        Types.Fr memory delta_4 = proof.wire_values_at_z[0].add_fr(proof.wire_values_at_z[0]);
        delta_4.add_assign(delta_4);
        delta_4.neg_assign();
        delta_4.add_assign(proof.wire_values_at_z_omega[3]);

        Types.Fr memory t0 = delta_1.sqr_fr();
        t0.sub_assign(delta_1);
        Types.Fr memory t1 = delta_1.sub_fr(Types.Fr(2));
        t0.mul_assign(t1);
        t1 = delta_1.sub_fr(Types.Fr(3));
        t0.mul_assign(t1);
        t0.mul_assign(alpha_powers[0]); // TODO CHECK
        Types.Fr memory range_accumulator = PairingsBn254.copy(t0);

        t0 = delta_2.sqr_fr();
        t0.sub_assign(delta_2);
        t1 = delta_2.sub_fr(Types.Fr(2));
        t0.mul_assign(t1);
        t1 = delta_2.sub_fr(Types.Fr(3));
        t0.mul_assign(t1);
        t0.mul_assign(alpha_powers[1]);
        range_accumulator.add_assign(t0);

        t0 = delta_3.sqr_fr();
        t0.sub_assign(delta_3);
        t1 = delta_3.sub_fr(Types.Fr(2));
        t0.mul_assign(t1);
        t1 = delta_3.sub_fr(Types.Fr(3));
        t0.mul_assign(t1);
        t0.mul_assign(alpha_powers[2]);
        range_accumulator.add_assign(t0);

        t0 = delta_4.sqr_fr();
        t0.sub_assign(delta_4);
        t1 = delta_4.sub_fr(Types.Fr(2));
        t0.mul_assign(t1);
        t1 = delta_4.sub_fr(Types.Fr(3));
        t0.mul_assign(t1);
        t0.mul_assign(alpha_powers[3]);
        range_accumulator.add_assign(t0);

        range_accumulator.mul_assign(challenges.v[10]);

        Types.G1Point memory kate_component = PairingsBn254.point_mul(vk.QRANGE, range_accumulator);

        challenges.alpha_base = alpha_powers[3].mul_fr(challenges.alpha);

        return (kate_component, range_accumulator);
    }

    // Compute grand product opening scalar and perform kate verification scalar multiplication
    function compute_grand_product_opening_scalar(
        Types.Proof memory proof,
        Types.VerificationKey memory vk,
        Types.ChallengeTranscript memory challenges,
        Types.Fr memory L1_fr
    ) public view returns (Types.G1Point memory, Types.Fr[2] memory) {
        Types.Fr[2] memory partial_grand_product;
        partial_grand_product[0] = PairingsBn254.mul_fr(challenges.beta, challenges.zeta);
        partial_grand_product[0].add_assign(proof.wire_values_at_z[0]);
        partial_grand_product[0].add_assign(challenges.gamma);

        Types.Fr memory t0 = PairingsBn254.mul_fr(vk.permutation_non_residues[0], challenges.zeta);
        t0.mul_assign(challenges.beta);
        t0.add_assign(challenges.gamma);
        t0.add_assign(proof.wire_values_at_z[1]);
        partial_grand_product[0].mul_assign(t0);

        t0 = PairingsBn254.mul_fr(vk.permutation_non_residues[1], challenges.zeta);
        t0.mul_assign(challenges.beta);
        t0.add_assign(challenges.gamma);
        t0.add_assign(proof.wire_values_at_z[2]);
        partial_grand_product[0].mul_assign(t0);

        t0 = PairingsBn254.mul_fr(vk.permutation_non_residues[2], challenges.zeta);
        t0.mul_assign(challenges.beta);
        t0.add_assign(challenges.gamma);
        t0.add_assign(proof.wire_values_at_z[3]);
        partial_grand_product[0].mul_assign(t0);

        partial_grand_product[0].mul_assign(challenges.alpha_base);

        Types.Fr memory alpha_cubed = PairingsBn254.mul_fr(challenges.alpha_base, challenges.alpha);
        alpha_cubed.mul_assign(challenges.alpha);

        Types.Fr memory t1 = L1_fr.mul_fr(alpha_cubed);

        partial_grand_product[0].add_assign(t1);
        partial_grand_product[0].mul_assign(challenges.v[10]);
        partial_grand_product[0].add_assign(challenges.u);

        Types.G1Point memory accumulator = PairingsBn254.point_mul(
            proof.grand_product_commitment,
            partial_grand_product[0]
        );

        {
            Types.Fr[3] memory sigma_terms;
            sigma_terms[0] = proof.permutation_polynomials_at_z[0].mul_fr(challenges.beta);
            sigma_terms[0].add_assign(challenges.gamma);
            sigma_terms[0].add_assign(proof.wire_values_at_z[0]);

            sigma_terms[1] = proof.permutation_polynomials_at_z[1].mul_fr(challenges.beta);
            sigma_terms[1].add_assign(challenges.gamma);
            sigma_terms[1].add_assign(proof.wire_values_at_z[1]);

            sigma_terms[2] = proof.permutation_polynomials_at_z[2].mul_fr(challenges.beta);
            sigma_terms[2].add_assign(challenges.gamma);
            sigma_terms[2].add_assign(proof.wire_values_at_z[2]);

            partial_grand_product[1] = sigma_terms[0].mul_fr(sigma_terms[1]);
            partial_grand_product[1].mul_assign(sigma_terms[2]);
            partial_grand_product[1].mul_assign(proof.grand_product_at_z_omega);
            partial_grand_product[1].mul_assign(challenges.alpha_base);
            partial_grand_product[1].neg_assign();
            partial_grand_product[1].mul_assign(challenges.beta);
            partial_grand_product[1].mul_assign(challenges.v[10]);

            Types.G1Point memory S = PairingsBn254.point_mul(vk.sigma_commitments[3], partial_grand_product[1]);
            accumulator.point_add_assign(S);
        }
        challenges.alpha_base = PairingsBn254.mul_fr(alpha_cubed, challenges.alpha);
        return (accumulator, partial_grand_product);
    }
}





/**
 * @title Challenge transcript library
 * @dev Used to collect the data necessary to calculate the various challenges: beta, gamma, alpha, zeta, nu[7], u
 */
library TranscriptLibrary {
    uint256 constant r_mod = 21888242871839275222246405745257275088548364400416034343698204186575808495617;

    struct Transcript {
        bytes32 current_challenge;
        bytes data;
        uint32 challenge_counter;
        bytes32 debug_data;
    }
    event ChallengeDebug(bytes32 data);

    /**
     * Instantiate a transcript and calculate the initial challenge, from which other challenges are derived.
     *
     * Resembles the preamble round in the Plonk prover
     */
    function new_transcript(uint256 circuit_size, uint256 num_public_inputs)
        internal
        pure
        returns (Transcript memory transcript)
    {
        bytes memory formatted_circuit_size = format_4_byte_variable(uint32(circuit_size));
        bytes memory formatted_num_public_inputs = format_4_byte_variable(uint32(num_public_inputs));

        transcript.current_challenge = keccak256(abi.encodePacked(formatted_circuit_size, formatted_num_public_inputs));
        transcript.debug_data = transcript.current_challenge;
        transcript.data = abi.encodePacked(transcript.current_challenge);
        transcript.challenge_counter = 0;
    }

    function format_4_byte_variable(uint32 input) internal pure returns (bytes memory) {
        // uint8 byte0 = uint8(input & 0xff);
        // uint8 byte1 = uint8((input >> 8) & 0xff);
        // uint8 byte2 = uint8((input >> 16) & 0xff);
        // uint8 byte3 = uint8((input >> 24) & 0xff);
        // // TODO SWAP
        uint8 byte0 = uint8((input >> 24) & 0xff);
        uint8 byte1 = uint8((input >> 16) & 0xff);
        uint8 byte2 = uint8((input >> 8) & 0xff);
        uint8 byte3 = uint8((input) & 0xff);
        return abi.encodePacked(byte0, byte1, byte2, byte3);
    }

    /**
     * Add a uint256 into the transcript
     */
    function update_with_u256(Transcript memory self, uint256 value) internal pure {
        self.data = abi.encodePacked(self.data, value);
    }

    /**
     * Add a field element into the transcript
     */
    function update_with_fr(Transcript memory self, Types.Fr memory value) internal pure {
        update_with_u256(self, value.value);
    }

    /**
     * Add a g1 point into the transcript
     */
    function update_with_g1(Transcript memory self, Types.G1Point memory p) internal pure {
        // in the C++ prover, the y coord is appended first before the x
        update_with_u256(self, p.Y);
        update_with_u256(self, p.X);
    }

    /**
     * Append byte
     */
    function append_byte(Transcript memory self, uint8 value) internal pure {
        self.data = abi.encodePacked(self.data, value);
    }

    /**
     * Draw a challenge
     */
    function get_challenge(Transcript memory self) internal pure returns (Types.Fr memory) {
        bytes32 challenge = keccak256(self.data);
        self.current_challenge = challenge;
        self.data = abi.encodePacked(challenge);
        return Types.Fr({value: uint256(challenge) % r_mod});
    }
}



