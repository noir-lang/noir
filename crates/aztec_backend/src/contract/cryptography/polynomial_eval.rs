#[macro_export]
macro_rules! POLYNOMIALEVAL_LIBRARY {
    () => { r#"

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
    
    
    "#};
}
