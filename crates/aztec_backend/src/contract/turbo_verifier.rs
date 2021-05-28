use crate::contract::cryptography::*;
use regex::{Captures, Regex};

/// Places the verification method which contains the verification key
/// into the TurboVerifier contract
// XXX: Regex seems to be really slow. We are also allocating when we call push_str, it is probably more efficient to write &str straight to file
pub fn create(vk_method: &str) -> String {
    template_replace(TURBOVERIFIER, &[vk_method])
}

// Regex code taken from: https://stackoverflow.com/questions/53974404/replacing-numbered-placeholders-with-elements-of-a-vector-in-rust
fn template_replace(template: &str, values: &[&str]) -> String {
    let regex = Regex::new(r#"\$(\d+)"#).unwrap();
    let mut turbo_verifier_contract = regex
        .replace_all(template, |captures: &Captures| {
            values.get(index(captures)).unwrap_or(&"")
        })
        .to_string();

    turbo_verifier_contract.push_str(cryptography_libraries());

    turbo_verifier_contract
}

fn index(captures: &Captures) -> usize {
    captures.get(1).unwrap().as_str().parse().unwrap()
}

const TURBOVERIFIER: &str = r#"
// SPDX-License-Identifier: GPL-2.0-only
// Copyright 2020 Spilsbury Holdings Ltd

pragma solidity >=0.6.0 <0.8.0;
pragma experimental ABIEncoderV2;

/**
 * @title Turbo Plonk proof verification contract
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
    using Bn254Crypto for Types.G1Point;
    using Bn254Crypto for Types.G2Point;
    using TranscriptLibrary for TranscriptLibrary.Transcript;

    function verify(bytes calldata, bytes calldata) public view returns (bool) {
        // extract the correct rollup verification key
        Types.VerificationKey memory vk = get_verification_key();
        uint256 num_public_inputs = vk.num_inputs;

        // parse the input calldata and construct a Proof object
        Types.Proof memory decoded_proof = deserialize_proof(num_public_inputs, vk);

        TranscriptLibrary.Transcript memory transcript = TranscriptLibrary.new_transcript(
            vk.circuit_size,
            vk.num_inputs
        );

        // reconstruct the beta, gamma, alpha and zeta challenges
        Types.ChallengeTranscript memory challenges;
        transcript.get_beta_gamma_challenges(challenges, vk.num_inputs);
        transcript.update_with_g1(decoded_proof.Z);
        challenges.alpha = transcript.get_challenge();
        challenges.alpha_base = challenges.alpha;
        transcript.update_with_four_g1_elements(
            decoded_proof.T1,
            decoded_proof.T2,
            decoded_proof.T3,
            decoded_proof.T4
        );
        challenges.zeta = transcript.get_challenge();


        /**
         * Compute all inverses that will be needed throughout the program here.
         *
         * This is an efficiency improvement - it allows us to make use of the batch inversion Montgomery trick,
         * which allows all inversions to be replaced with one inversion operation, at the expense of a few
         * additional multiplications
         **/
        (uint256 quotient_eval, uint256 L1) = evalaute_field_operations(decoded_proof, vk, challenges);
        decoded_proof.quotient_polynomial_eval = quotient_eval;

        // reconstruct the nu and u challenges
        transcript.get_nu_challenges(decoded_proof.quotient_polynomial_eval, challenges);
        transcript.update_with_g1(decoded_proof.PI_Z);
        transcript.update_with_g1(decoded_proof.PI_Z_OMEGA);
        challenges.u = transcript.get_challenge();

        //reset 'alpha base'
        challenges.alpha_base = challenges.alpha;

        Types.G1Point memory partial_opening_commitment = PolynomialEval.compute_linearised_opening_terms(
            challenges,
            L1,
            vk,
            decoded_proof
        );

        Types.G1Point memory batch_opening_commitment = PolynomialEval.compute_batch_opening_commitment(
            challenges,
            vk,
            partial_opening_commitment,
            decoded_proof
        );

        uint256 batch_evaluation_g1_scalar = PolynomialEval.compute_batch_evaluation_scalar_multiplier(
            decoded_proof,
            challenges
        );

        bool result = perform_pairing(
            batch_opening_commitment,
            batch_evaluation_g1_scalar,
            challenges,
            decoded_proof,
            vk
        );
        return result;
    }

    $0

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
     * and then we perform all necessary inversions in one go at the end of `evalaute_field_operations()`. This
     * gives us the various variables that need to be returned.
     *
     * @param decoded_proof - deserialised proof
     * @param vk - verification key
     * @param challenges - all challenges (alpha, beta, gamma, zeta, nu[NUM_NU_CHALLENGES], u) stored in
     * ChallengeTranscript struct form
     * @return quotient polynomial evaluation (field element) and lagrange 1 evaluation (field element)
     */
    function evalaute_field_operations(
        Types.Proof memory decoded_proof,
        Types.VerificationKey memory vk,
        Types.ChallengeTranscript memory challenges
    ) internal view returns (uint256, uint256) {
        uint256 public_input_delta;
        uint256 zero_polynomial_eval;
        uint256 l_start;
        uint256 l_end;
        {
            (uint256 public_input_numerator, uint256 public_input_denominator) = PolynomialEval.compute_public_input_delta(
                challenges,
                vk
            );

            (
                uint256 vanishing_numerator,
                uint256 vanishing_denominator,
                uint256 lagrange_numerator,
                uint256 l_start_denominator,
                uint256 l_end_denominator
            ) = PolynomialEval.compute_lagrange_and_vanishing_fractions(vk, challenges.zeta);


            (zero_polynomial_eval, public_input_delta, l_start, l_end) = PolynomialEval.compute_batch_inversions(
                public_input_numerator,
                public_input_denominator,
                vanishing_numerator,
                vanishing_denominator,
                lagrange_numerator,
                l_start_denominator,
                l_end_denominator
            );
        }

        uint256 quotient_eval = PolynomialEval.compute_quotient_polynomial(
            zero_polynomial_eval,
            public_input_delta,
            challenges,
            l_start,
            l_end,
            decoded_proof
        );

        return (quotient_eval, l_start);
    }


    /**
     * @dev Perform the pairing check
     * @param batch_opening_commitment - G1 point representing the calculated batch opening commitment
     * @param batch_evaluation_g1_scalar - uint256 representing the batch evaluation scalar multiplier to be applied to the G1 generator point
     * @param challenges - all challenges (alpha, beta, gamma, zeta, nu[NUM_NU_CHALLENGES], u) stored in
     * ChallengeTranscript struct form
     * @param vk - verification key
     * @param decoded_proof - deserialised proof
     * @return bool specifying whether the pairing check was successful
     */
    function perform_pairing(
        Types.G1Point memory batch_opening_commitment,
        uint256 batch_evaluation_g1_scalar,
        Types.ChallengeTranscript memory challenges,
        Types.Proof memory decoded_proof,
        Types.VerificationKey memory vk
    ) internal view returns (bool) {

        uint256 u = challenges.u;
        bool success;
        uint256 p = Bn254Crypto.r_mod;
        Types.G1Point memory rhs;     
        Types.G1Point memory PI_Z_OMEGA = decoded_proof.PI_Z_OMEGA;
        Types.G1Point memory PI_Z = decoded_proof.PI_Z;
        PI_Z.validateG1Point();
        PI_Z_OMEGA.validateG1Point();
    
        // rhs = zeta.[PI_Z] + u.zeta.omega.[PI_Z_OMEGA] + [batch_opening_commitment] - batch_evaluation_g1_scalar.[1]
        // scope this block to prevent stack depth errors
        {
            uint256 zeta = challenges.zeta;
            uint256 pi_z_omega_scalar = vk.work_root;
            assembly {
                pi_z_omega_scalar := mulmod(pi_z_omega_scalar, zeta, p)
                pi_z_omega_scalar := mulmod(pi_z_omega_scalar, u, p)
                batch_evaluation_g1_scalar := sub(p, batch_evaluation_g1_scalar)

                // store accumulator point at mptr
                let mPtr := mload(0x40)

                // set accumulator = batch_opening_commitment
                mstore(mPtr, mload(batch_opening_commitment))
                mstore(add(mPtr, 0x20), mload(add(batch_opening_commitment, 0x20)))

                // compute zeta.[PI_Z] and add into accumulator
                mstore(add(mPtr, 0x40), mload(PI_Z))
                mstore(add(mPtr, 0x60), mload(add(PI_Z, 0x20)))
                mstore(add(mPtr, 0x80), zeta)
                success := staticcall(gas(), 7, add(mPtr, 0x40), 0x60, add(mPtr, 0x40), 0x40)
                success := and(success, staticcall(gas(), 6, mPtr, 0x80, mPtr, 0x40))

                // compute u.zeta.omega.[PI_Z_OMEGA] and add into accumulator
                mstore(add(mPtr, 0x40), mload(PI_Z_OMEGA))
                mstore(add(mPtr, 0x60), mload(add(PI_Z_OMEGA, 0x20)))
                mstore(add(mPtr, 0x80), pi_z_omega_scalar)
                success := and(success, staticcall(gas(), 7, add(mPtr, 0x40), 0x60, add(mPtr, 0x40), 0x40))
                success := and(success, staticcall(gas(), 6, mPtr, 0x80, mPtr, 0x40))

                // compute -batch_evaluation_g1_scalar.[1]
                mstore(add(mPtr, 0x40), 0x01) // hardcoded generator point (1, 2)
                mstore(add(mPtr, 0x60), 0x02)
                mstore(add(mPtr, 0x80), batch_evaluation_g1_scalar)
                success := and(success, staticcall(gas(), 7, add(mPtr, 0x40), 0x60, add(mPtr, 0x40), 0x40))

                // add -batch_evaluation_g1_scalar.[1] and the accumulator point, write result into rhs
                success := and(success, staticcall(gas(), 6, mPtr, 0x80, rhs, 0x40))
            }
        }

        Types.G1Point memory lhs;   
        assembly {
            // store accumulator point at mptr
            let mPtr := mload(0x40)

            // copy [PI_Z] into mPtr
            mstore(mPtr, mload(PI_Z))
            mstore(add(mPtr, 0x20), mload(add(PI_Z, 0x20)))

            // compute u.[PI_Z_OMEGA] and write to (mPtr + 0x40)
            mstore(add(mPtr, 0x40), mload(PI_Z_OMEGA))
            mstore(add(mPtr, 0x60), mload(add(PI_Z_OMEGA, 0x20)))
            mstore(add(mPtr, 0x80), u)
            success := and(success, staticcall(gas(), 7, add(mPtr, 0x40), 0x60, add(mPtr, 0x40), 0x40))
            
            // add [PI_Z] + u.[PI_Z_OMEGA] and write result into lhs
            success := and(success, staticcall(gas(), 6, mPtr, 0x80, lhs, 0x40))
        }

        // negate lhs y-coordinate
        uint256 q = Bn254Crypto.p_mod;
        assembly {
            mstore(add(lhs, 0x20), sub(q, mload(add(lhs, 0x20))))
        }

        if (vk.contains_recursive_proof)
        {
            // If the proof itself contains an accumulated proof,
            // we will have extracted two G1 elements `recursive_P1`, `recursive_p2` from the public inputs

            // We need to evaluate that e(recursive_P1, [x]_2) == e(recursive_P2, [1]_2) to finish verifying the inner proof
            // We do this by creating a random linear combination between (lhs, recursive_P1) and (rhs, recursivee_P2)
            // That way we still only need to evaluate one pairing product

            // We use `challenge.u * challenge.u` as the randomness to create a linear combination
            // challenge.u is produced by hashing the entire transcript, which contains the public inputs (and by extension the recursive proof)

            // i.e. [lhs] = [lhs] + u.u.[recursive_P1]
            //      [rhs] = [rhs] + u.u.[recursive_P2]
            Types.G1Point memory recursive_P1 = decoded_proof.recursive_P1;
            Types.G1Point memory recursive_P2 = decoded_proof.recursive_P2;
            recursive_P1.validateG1Point();
            recursive_P2.validateG1Point();
            assembly {
                let mPtr := mload(0x40)

                // compute u.u.[recursive_P1]
                mstore(mPtr, mload(recursive_P1))
                mstore(add(mPtr, 0x20), mload(add(recursive_P1, 0x20)))
                mstore(add(mPtr, 0x40), mulmod(u, u, p)) // separator_challenge = u * u
                success := and(success, staticcall(gas(), 7, mPtr, 0x60, add(mPtr, 0x60), 0x40))

                // compute u.u.[recursive_P2] (u*u is still in memory at (mPtr + 0x40), no need to re-write it)
                mstore(mPtr, mload(recursive_P2))
                mstore(add(mPtr, 0x20), mload(add(recursive_P2, 0x20)))
                success := and(success, staticcall(gas(), 7, mPtr, 0x60, mPtr, 0x40))

                // compute u.u.[recursiveP2] + rhs and write into rhs
                mstore(add(mPtr, 0xa0), mload(rhs))
                mstore(add(mPtr, 0xc0), mload(add(rhs, 0x20)))
                success := and(success, staticcall(gas(), 6, add(mPtr, 0x60), 0x80, rhs, 0x40))

                // compute u.u.[recursiveP1] + lhs and write into lhs
                mstore(add(mPtr, 0x40), mload(lhs))
                mstore(add(mPtr, 0x60), mload(add(lhs, 0x20)))
                success := and(success, staticcall(gas(), 6, mPtr, 0x80, lhs, 0x40))
            }
        }

        require(success, "perform_pairing G1 operations preamble fail");

        return Bn254Crypto.pairingProd2(rhs, Bn254Crypto.P2(), lhs, vk.g2_x);
    }

    /**
     * @dev Deserialize a proof into a Proof struct
     * @param num_public_inputs - number of public inputs in the proof. Taken from verification key
     * @return proof - proof deserialized into the proof struct
     */
    function deserialize_proof(uint256 num_public_inputs, Types.VerificationKey memory vk)
        internal
        pure
        returns (Types.Proof memory proof)
    {
        uint256 p = Bn254Crypto.r_mod;
        uint256 q = Bn254Crypto.p_mod;
        uint256 data_ptr;
        uint256 proof_ptr;
        // first 32 bytes of bytes array contains length, skip it
        assembly {
            data_ptr := add(calldataload(0x04), 0x24)
            proof_ptr := proof
        }

        if (vk.contains_recursive_proof) {
            uint256 index_counter = vk.recursive_proof_indices * 32;
            uint256 x0 = 0;
            uint256 y0 = 0;
            uint256 x1 = 0;
            uint256 y1 = 0;
            assembly {
                index_counter := add(index_counter, add(calldataload(0x24), 0x04))
                x0 := calldataload(index_counter)
                x0 := add(x0, shl(68, calldataload(add(index_counter, 0x20))))
                x0 := add(x0, shl(136, calldataload(add(index_counter, 0x40))))
                x0 := add(x0, shl(204, calldataload(add(index_counter, 0x60))))
                y0 := calldataload(add(index_counter, 0x80))
                y0 := add(y0, shl(68, calldataload(add(index_counter, 0xa0))))
                y0 := add(y0, shl(136, calldataload(add(index_counter, 0xc0))))
                y0 := add(y0, shl(204, calldataload(add(index_counter, 0xe0))))
                x1 := calldataload(add(index_counter, 0x100))
                x1 := add(x1, shl(68, calldataload(add(index_counter, 0x120))))
                x1 := add(x1, shl(136, calldataload(add(index_counter, 0x140))))
                x1 := add(x1, shl(204, calldataload(add(index_counter, 0x160))))
                y1 := calldataload(add(index_counter, 0x180))
                y1 := add(y1, shl(68, calldataload(add(index_counter, 0x1a0))))
                y1 := add(y1, shl(136, calldataload(add(index_counter, 0x1c0))))
                y1 := add(y1, shl(204, calldataload(add(index_counter, 0x1e0))))
            }

            proof.recursive_P1 = Bn254Crypto.new_g1(x0, y0);
            proof.recursive_P2 = Bn254Crypto.new_g1(x1, y1);
        }

        assembly {  
            // proof.W1
            mstore(mload(proof_ptr), mod(calldataload(add(data_ptr, 0x20)), q))
            mstore(add(mload(proof_ptr), 0x20), mod(calldataload(data_ptr), q))

            // proof.W2
            mstore(mload(add(proof_ptr, 0x20)), mod(calldataload(add(data_ptr, 0x60)), q))
            mstore(add(mload(add(proof_ptr, 0x20)), 0x20), mod(calldataload(add(data_ptr, 0x40)), q))
 
            // proof.W3
            mstore(mload(add(proof_ptr, 0x40)), mod(calldataload(add(data_ptr, 0xa0)), q))
            mstore(add(mload(add(proof_ptr, 0x40)), 0x20), mod(calldataload(add(data_ptr, 0x80)), q))

            // proof.W4
            mstore(mload(add(proof_ptr, 0x60)), mod(calldataload(add(data_ptr, 0xe0)), q))
            mstore(add(mload(add(proof_ptr, 0x60)), 0x20), mod(calldataload(add(data_ptr, 0xc0)), q))
  
            // proof.Z
            mstore(mload(add(proof_ptr, 0x80)), mod(calldataload(add(data_ptr, 0x120)), q))
            mstore(add(mload(add(proof_ptr, 0x80)), 0x20), mod(calldataload(add(data_ptr, 0x100)), q))
  
            // proof.T1
            mstore(mload(add(proof_ptr, 0xa0)), mod(calldataload(add(data_ptr, 0x160)), q))
            mstore(add(mload(add(proof_ptr, 0xa0)), 0x20), mod(calldataload(add(data_ptr, 0x140)), q))

            // proof.T2
            mstore(mload(add(proof_ptr, 0xc0)), mod(calldataload(add(data_ptr, 0x1a0)), q))
            mstore(add(mload(add(proof_ptr, 0xc0)), 0x20), mod(calldataload(add(data_ptr, 0x180)), q))

            // proof.T3
            mstore(mload(add(proof_ptr, 0xe0)), mod(calldataload(add(data_ptr, 0x1e0)), q))
            mstore(add(mload(add(proof_ptr, 0xe0)), 0x20), mod(calldataload(add(data_ptr, 0x1c0)), q))

            // proof.T4
            mstore(mload(add(proof_ptr, 0x100)), mod(calldataload(add(data_ptr, 0x220)), q))
            mstore(add(mload(add(proof_ptr, 0x100)), 0x20), mod(calldataload(add(data_ptr, 0x200)), q))
  
            // proof.w1 to proof.w4
            mstore(add(proof_ptr, 0x120), mod(calldataload(add(data_ptr, 0x240)), p))
            mstore(add(proof_ptr, 0x140), mod(calldataload(add(data_ptr, 0x260)), p))
            mstore(add(proof_ptr, 0x160), mod(calldataload(add(data_ptr, 0x280)), p))
            mstore(add(proof_ptr, 0x180), mod(calldataload(add(data_ptr, 0x2a0)), p))
 
            // proof.sigma1
            mstore(add(proof_ptr, 0x1a0), mod(calldataload(add(data_ptr, 0x2c0)), p))

            // proof.sigma2
            mstore(add(proof_ptr, 0x1c0), mod(calldataload(add(data_ptr, 0x2e0)), p))

            // proof.sigma3
            mstore(add(proof_ptr, 0x1e0), mod(calldataload(add(data_ptr, 0x300)), p))

            // proof.q_arith
            mstore(add(proof_ptr, 0x200), mod(calldataload(add(data_ptr, 0x320)), p))

            // proof.q_ecc
            mstore(add(proof_ptr, 0x220), mod(calldataload(add(data_ptr, 0x340)), p))

            // proof.q_c
            mstore(add(proof_ptr, 0x240), mod(calldataload(add(data_ptr, 0x360)), p))
 
            // proof.linearization_polynomial
            mstore(add(proof_ptr, 0x260), mod(calldataload(add(data_ptr, 0x380)), p))

            // proof.grand_product_at_z_omega
            mstore(add(proof_ptr, 0x280), mod(calldataload(add(data_ptr, 0x3a0)), p))

            // proof.w1_omega to proof.w4_omega
            mstore(add(proof_ptr, 0x2a0), mod(calldataload(add(data_ptr, 0x3c0)), p))
            mstore(add(proof_ptr, 0x2c0), mod(calldataload(add(data_ptr, 0x3e0)), p))
            mstore(add(proof_ptr, 0x2e0), mod(calldataload(add(data_ptr, 0x400)), p))
            mstore(add(proof_ptr, 0x300), mod(calldataload(add(data_ptr, 0x420)), p))
  
            // proof.PI_Z
            mstore(mload(add(proof_ptr, 0x320)), mod(calldataload(add(data_ptr, 0x460)), q))
            mstore(add(mload(add(proof_ptr, 0x320)), 0x20), mod(calldataload(add(data_ptr, 0x440)), q))

            // proof.PI_Z_OMEGA
            mstore(mload(add(proof_ptr, 0x340)), mod(calldataload(add(data_ptr, 0x4a0)), q))
            mstore(add(mload(add(proof_ptr, 0x340)), 0x20), mod(calldataload(add(data_ptr, 0x480)), q))
        }
    }
}

"#;
