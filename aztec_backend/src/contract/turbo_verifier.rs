

use crate::contract::cryptography::*;
use regex::{Regex, Captures};

/// Places the verification method which contains the verification key
/// into the TurboVerifier contract
// XXX: Regex seems to be really slow. We are also allocating when we call push_str, it is probably more efficient to write &str straight to file  
pub fn create(vk_method : &str) -> String {
  template_replace(TURBOVERIFIER, &[vk_method])
}

// Regex code taken from: https://stackoverflow.com/questions/53974404/replacing-numbered-placeholders-with-elements-of-a-vector-in-rust
fn template_replace(template: &str, values: &[&str]) -> String {
  let regex = Regex::new(r#"\$(\d+)"#).unwrap();
  let mut turbo_verifier_contract = regex.replace_all(template, |captures: &Captures| {
      values
          .get(index(captures))
          .unwrap_or(&"")
  }).to_string();

  turbo_verifier_contract.push_str(cryptography_libraries());

  turbo_verifier_contract
}

fn index(captures: &Captures) -> usize {
  captures.get(1)
      .unwrap()
      .as_str()
      .parse()
      .unwrap()
}


const TURBOVERIFIER : &str = r#"
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

    $0
}
"#;